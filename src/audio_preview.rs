use crate::song::Song;
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};
use std::{
    fs::File,
    path::PathBuf,
    sync::{
        Arc,
        mpsc::{self, Receiver, Sender, TryRecvError},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

const MAX_AUDIO_BYTES: u64 = 2 * 1024 * 1024;
const MAX_SAMPLES_PER_NOTE: usize = 2_000_000;
const MAX_SAMPLES_PER_INSTRUMENT: usize = 24_000_000;
const MAX_VOICES: usize = 48;
const MAX_NOTE_LATENESS_MS: f64 = 120.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Instrument {
    #[default]
    Guitar,
    Harp,
}

impl Instrument {
    pub const ALL: [Self; 2] = [Self::Guitar, Self::Harp];

    pub fn code(self) -> &'static str {
        match self {
            Self::Guitar => "guitar",
            Self::Harp => "harp",
        }
    }

    pub fn folder(self) -> &'static str {
        match self {
            Self::Guitar => "Guitar",
            Self::Harp => "Harp",
        }
    }

    pub fn translation_key(self) -> &'static str {
        self.code()
    }

    pub fn audio_extension(self) -> &'static str {
        match self {
            Self::Guitar => "ogg",
            Self::Harp => "wav",
        }
    }

    pub fn semitones(self, key: usize) -> i8 {
        const HARP_OFFSETS: [i8; 15] =
            [0, 0, 0, -1, -1, -1, -1, 0, 0, 0, -1, -1, -1, -1, 0];
        if self == Self::Harp {
            HARP_OFFSETS.get(key).copied().unwrap_or(0)
        } else {
            0
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|i| i.code() == code)
    }
}

#[derive(Clone)]
struct Sample {
    values: Arc<[f32]>,
    channels: u16,
    rate: u32,
    semitones: i8,
}

impl Sample {
    fn read(path: &std::path::Path, semitones: i8) -> Result<Self, String> {
        let reader = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
        if reader.metadata().map_err(|e| e.to_string())?.len() > MAX_AUDIO_BYTES {
            return Err(format!("Audio sample too large: {}", path.display()));
        }
        let decoder = Decoder::try_from(reader)
            .map_err(|e| format!("{}: {e}", path.display()))?;
        let channels = decoder.channels();
        let rate = decoder.sample_rate();
        let values: Vec<f32> = decoder.take(MAX_SAMPLES_PER_NOTE + 1).collect();
        if channels == 0 || rate == 0 || values.len() < channels as usize
            || values.len() > MAX_SAMPLES_PER_NOTE
        {
            return Err(format!("Audio PCM duration/size exceeds limit: {}", path.display()));
        }
        Ok(Self { values: values.into(), channels, rate, semitones })
    }

    fn voice(&self) -> SampleVoice {
        SampleVoice { sample: self.clone(), offset: 0 }
    }
}

struct SampleVoice {
    sample: Sample,
    offset: usize,
}

impl Iterator for SampleVoice {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let next = self.sample.values.get(self.offset).copied();
        if next.is_some() {
            self.offset += 1;
        }
        next
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.sample.values.len().saturating_sub(self.offset);
        (remaining, Some(remaining))
    }
}

impl Source for SampleVoice {
    fn current_span_len(&self) -> Option<usize> {
        Some(self.sample.values.len().saturating_sub(self.offset))
    }

    fn channels(&self) -> u16 {
        self.sample.channels
    }

    fn sample_rate(&self) -> u32 {
        (self.sample.rate as f64 * 2.0_f64.powf(self.sample.semitones as f64 / 12.0))
            .round().max(1.0) as u32
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f64(
            self.sample.values.len() as f64
                / (self.sample.channels as f64
                    * self.sample.rate as f64
                    * 2.0_f64.powf(self.sample.semitones as f64 / 12.0)),
        ))
    }
}

struct ScheduledPlayback {
    song: Arc<Song>,
    next_note: usize,
    position_ms: f64,
    started_at: Instant,
    speed: f32,
}

impl ScheduledPlayback {
    fn new(song: Arc<Song>, position_ms: u64, speed: f32, started_at: Instant) -> Self {
        let next_note = song.notes.partition_point(|note| note.time_ms < position_ms);
        Self {
            song,
            next_note,
            position_ms: position_ms as f64,
            started_at,
            speed: speed.clamp(0.5, 2.0),
        }
    }

    fn position_at(&self, now: Instant) -> f64 {
        self.position_ms
            + now.saturating_duration_since(self.started_at).as_secs_f64()
                * 1000.0 * self.speed as f64
    }

    fn resync(&mut self, position_ms: u64, speed: f32, now: Instant) {
        self.position_ms = position_ms as f64;
        self.started_at = now;
        self.speed = speed.clamp(0.5, 2.0);
        self.next_note = self.song.notes.partition_point(|note| note.time_ms < position_ms);
    }

    fn wait_until_next(&self, now: Instant) -> Duration {
        let Some(note) = self.song.notes.get(self.next_note) else {
            return Duration::from_millis(40);
        };
        let remaining = (note.time_ms as f64 - self.position_at(now)).max(0.0);
        Duration::from_secs_f64(remaining / self.speed as f64 / 1000.0)
            .min(Duration::from_millis(40))
    }
}

enum AudioCommand {
    Samples(Instrument, Arc<Vec<Sample>>),
    Select(Instrument),
    Play(ScheduledPlayback),
    Stop,
    Resync { position_ms: u64, speed: f32, now: Instant },
    Shutdown,
}

struct AudioWorker {
    instrument: Instrument,
    samples: Option<Arc<Vec<Sample>>>,
    playback: Option<ScheduledPlayback>,
    output: Option<OutputStream>,
    voices: Vec<Sink>,
    errors: Sender<String>,
}

impl AudioWorker {
    fn new(instrument: Instrument, errors: Sender<String>) -> Self {
        Self {
            instrument,
            samples: None,
            playback: None,
            output: None,
            voices: Vec::new(),
            errors,
        }
    }

    fn stop_voices(&mut self) {
        for sink in self.voices.drain(..) {
            sink.stop();
        }
    }

    fn handle(&mut self, command: AudioCommand) -> bool {
        match command {
            AudioCommand::Samples(instrument, samples) => {
                if instrument == self.instrument {
                    self.samples = Some(samples);
                }
            }
            AudioCommand::Select(instrument) => {
                self.stop_voices();
                self.playback = None;
                self.samples = None;
                self.instrument = instrument;
            }
            AudioCommand::Play(state) => {
                self.stop_voices();
                self.playback = Some(state);
            }
            AudioCommand::Stop => {
                self.playback = None;
                self.stop_voices();
            }
            AudioCommand::Resync { position_ms, speed, now } => {
                if let Some(state) = self.playback.as_mut() {
                    state.resync(position_ms, speed, now);
                }
            }
            AudioCommand::Shutdown => return false,
        }
        true
    }

    fn play_note(&mut self, key: u8) -> Result<(), String> {
        let Some(samples) = self.samples.as_ref() else {
            return Ok(());
        };
        let sample = samples.get(usize::from(key))
            .ok_or_else(|| format!("Invalid Sky note index {key}"))?;
        if self.output.is_none() {
            self.output = Some(
                OutputStreamBuilder::open_default_stream()
                    .map_err(|e| format!("Cannot open default audio device: {e}"))?,
            );
        }
        if self.voices.len() >= MAX_VOICES {
            self.voices.remove(0).stop();
        }
        let output = self.output.as_ref().expect("audio output ready");
        let sink = Sink::connect_new(output.mixer());
        sink.set_volume(0.60);
        sink.append(sample.voice());
        self.voices.push(sink);
        Ok(())
    }

    fn schedule(&mut self) -> Result<(), String> {
        if self.samples.is_none() {
            return Ok(());
        }
        let Some(playback) = self.playback.as_mut() else {
            return Ok(());
        };
        let position = playback.position_at(Instant::now());
        let mut due = Vec::new();
        while let Some(note) = playback.song.notes.get(playback.next_note) {
            if note.time_ms as f64 > position + 2.0 {
                break;
            }
            if position - (note.time_ms as f64) <= MAX_NOTE_LATENESS_MS {
                due.push(note.key);
            }
            playback.next_note += 1;
        }
        let done = playback.next_note == playback.song.notes.len()
            && position > playback.song.duration_ms as f64 + 120.0;
        if done {
            self.playback = None;
        }
        if due.is_empty() {
            return Ok(());
        }
        self.voices.retain(|voice| !voice.empty());
        for key in due {
            self.play_note(key)?;
        }
        Ok(())
    }

    fn run(mut self, receiver: Receiver<AudioCommand>) {
        loop {
            let wait = self.playback.as_ref()
                .filter(|_| self.samples.is_some())
                .map(|playing| playing.wait_until_next(Instant::now()))
                .unwrap_or(Duration::from_secs(30));
            match receiver.recv_timeout(wait) {
                Ok(command) => {
                    if !self.handle(command) {
                        break;
                    }
                    while let Ok(command) = receiver.try_recv() {
                        if !self.handle(command) {
                            self.stop_voices();
                            return;
                        }
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if let Err(error) = self.schedule() {
                        self.playback = None;
                        self.stop_voices();
                        let _ = self.errors.send(error);
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
        self.stop_voices();
    }
}

pub struct AudioPreview {
    instrument: Instrument,
    loaded: Option<Arc<Vec<Sample>>>,
    receiver: Option<Receiver<Result<Vec<Sample>, String>>>,
    commands: Sender<AudioCommand>,
    errors: Receiver<String>,
    worker: Option<JoinHandle<()>>,
    playback_active: bool,
    last_speed: f32,
}

impl AudioPreview {
    pub fn new(instrument: Instrument) -> Self {
        let (commands, commands_rx) = mpsc::channel();
        let (errors_tx, errors) = mpsc::channel();
        let worker = thread::Builder::new()
            .name("sky-audio-scheduler".into())
            .spawn(move || AudioWorker::new(instrument, errors_tx).run(commands_rx))
            .ok();
        let mut preview = Self {
            instrument,
            loaded: None,
            receiver: None,
            commands,
            errors,
            worker,
            playback_active: false,
            last_speed: 1.0,
        };
        preview.start_load();
        preview
    }

    fn directory() -> PathBuf {
        if let Some(folder) = std::env::current_exe().ok()
            .and_then(|exe| exe.parent().map(|base| base.join("instruments")))
        {
            if folder.is_dir() {
                return folder;
            }
        }
        #[cfg(debug_assertions)]
        {
            return PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("assets").join("instruments");
        }
        #[cfg(not(debug_assertions))]
        {
            PathBuf::from("instruments")
        }
    }

    fn load_samples(instrument: Instrument) -> Result<Vec<Sample>, String> {
        let directory = Self::directory().join(instrument.folder());
        let mut total = 0usize;
        let mut samples = Vec::with_capacity(15);
        for key in 0..15 {
            let sample = Sample::read(
                &directory.join(format!("{key}.{}", instrument.audio_extension())),
                instrument.semitones(key),
            )?;
            total = total.saturating_add(sample.values.len());
            if total > MAX_SAMPLES_PER_INSTRUMENT {
                return Err(format!(
                    "Audio sample bank {} exceeds safe decoded PCM memory budget ({total} samples)",
                    instrument.folder(),
                ));
            }
            samples.push(sample);
        }
        Ok(samples)
    }

    fn start_load(&mut self) {
        let instrument = self.instrument;
        let (sender, receiver) = mpsc::sync_channel(1);
        self.receiver = Some(receiver);
        let _ = thread::Builder::new()
            .name("sky-audio-sample-loader".into())
            .spawn(move || {
                let _ = sender.send(Self::load_samples(instrument));
            });
    }

    pub fn select(&mut self, instrument: Instrument) {
        if instrument == self.instrument {
            return;
        }
        self.stop();
        self.instrument = instrument;
        self.loaded = None;
        self.receiver = None;
        let _ = self.commands.send(AudioCommand::Select(instrument));
        self.start_load();
    }

    pub fn stop(&mut self) {
        self.playback_active = false;
        let _ = self.commands.send(AudioCommand::Stop);
    }

    pub fn play(&mut self, song: &Song, position_ms: u64, speed: f32) {
        self.playback_active = true;
        self.last_speed = speed;
        let state = ScheduledPlayback::new(
            Arc::new(song.clone()),
            position_ms,
            speed,
            Instant::now(),
        );
        let _ = self.commands.send(AudioCommand::Play(state));
    }

    pub fn update_speed(&mut self, position_ms: u64, speed: f32) {
        if self.playback_active && (speed - self.last_speed).abs() > 0.001 {
            self.last_speed = speed;
            let _ = self.commands.send(AudioCommand::Resync {
                position_ms,
                speed,
                now: Instant::now(),
            });
        }
    }

    pub fn poll(&mut self) -> Result<(), String> {
        if let Some(receiver) = self.receiver.as_ref() {
            match receiver.try_recv() {
                Ok(Ok(samples)) => {
                    let samples = Arc::new(samples);
                    let _ = self.commands.send(
                        AudioCommand::Samples(self.instrument, samples.clone()),
                    );
                    self.loaded = Some(samples);
                    self.receiver = None;
                }
                Ok(Err(error)) => {
                    self.receiver = None;
                    return Err(error);
                }
                Err(TryRecvError::Disconnected) => {
                    self.receiver = None;
                    return Err("Audio sample decoder disconnected".into());
                }
                Err(TryRecvError::Empty) => {}
            }
        }
        if let Ok(error) = self.errors.try_recv() {
            return Err(error);
        }
        if self.worker.is_none() {
            return Err("Audio scheduler could not start".into());
        }
        Ok(())
    }
}

impl Drop for AudioPreview {
    fn drop(&mut self) {
        let _ = self.commands.send(AudioCommand::Shutdown);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::song::Note;

    #[test]
    fn all_thirty_samples_decode_with_memory_limits() {
        if !AudioPreview::directory().join("Guitar").join("0.ogg").exists() {
            return;
        }
        for instrument in Instrument::ALL {
            assert_eq!(Instrument::from_code(instrument.code()), Some(instrument));
            let samples = AudioPreview::load_samples(instrument).unwrap();
            assert_eq!(samples.len(), 15);
            for sample in samples {
                assert!(sample.voice().total_duration().unwrap().as_millis() >= 50);
                assert!(sample.channels > 0 && sample.rate >= 8000);
            }
        }
        assert_eq!(Instrument::from_code("unknown"), None);
    }

    #[test]
    fn instrument_sample_formats_and_harp_pitch_mapping() {
        assert_eq!(Instrument::Harp.audio_extension(), "wav");
        assert_eq!(Instrument::Guitar.audio_extension(), "ogg");
        assert_eq!(Instrument::Harp.semitones(3), -1);
        assert_eq!(Instrument::Harp.semitones(6), -1);
        assert_eq!(Instrument::Harp.semitones(7), 0);
        assert_eq!(Instrument::Harp.semitones(14), 0);
        for key in 0..15 {
            assert_eq!(Instrument::Guitar.semitones(key), 0);
        }
    }

    #[test]
    fn scheduler_clock_advances_without_interface_frames() {
        let song = Arc::new(Song {
            name: "Timing".into(),
            notes: vec![
                Note { time_ms: 0, key: 0, hold_ms: 100 },
                Note { time_ms: 300, key: 1, hold_ms: 100 },
                Note { time_ms: 600, key: 2, hold_ms: 100 },
            ],
            duration_ms: 900,
        });
        let initial = Instant::now();
        let mut playing = ScheduledPlayback::new(song, 0, 1.0, initial);
        assert_eq!(playing.next_note, 0);
        assert!((playing.position_at(initial + Duration::from_millis(300)) - 300.0).abs() < 0.01);
        assert_eq!(playing.wait_until_next(initial), Duration::ZERO);
        playing.next_note = 1;
        assert!(playing.wait_until_next(initial + Duration::from_millis(250)) <= Duration::from_millis(40));
        playing.resync(450, 2.0, initial + Duration::from_millis(300));
        assert_eq!(playing.next_note, 2);
        assert!((playing.position_at(initial + Duration::from_millis(350)) - 550.0).abs() < 0.01);
    }

    #[test]
    fn idle_audio_preview_does_not_open_audio_device() {
        let mut preview = AudioPreview::new(Instrument::Harp);
        assert!(!preview.playback_active);
        preview.stop();
        preview.select(Instrument::Guitar);
        assert_eq!(preview.instrument, Instrument::Guitar);
    }
}
