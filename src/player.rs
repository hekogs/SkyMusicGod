use std::time::{Duration, Instant};
use crate::song::Song;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Status {
    Stopped,
    Armed,
    Playing,
    Paused,
    Finished,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct KeyEvent {
    pub key: u8,
    pub down: bool,
}

pub struct Player {
    pub status: Status,
    pub speed: f32,
    pub next_note: usize,
    started: Option<Instant>,
    elapsed_before: Duration,
    active_until: [Option<u64>; 15],
    delay_until: Option<Instant>,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            status: Status::Stopped,
            speed: 1.0,
            next_note: 0,
            started: None,
            elapsed_before: Duration::ZERO,
            active_until: [None; 15],
            delay_until: None,
        }
    }
}

impl Player {
    pub fn start(&mut self, delay: Duration) -> Vec<KeyEvent> {
        let released = self.release_all();
        self.next_note = 0;
        self.elapsed_before = Duration::ZERO;
        self.started = None;
        self.delay_until = if delay.is_zero() { None } else { Some(Instant::now() + delay) };
        self.status = if self.delay_until.is_some() { Status::Armed } else { Status::Playing };
        if self.status == Status::Playing {
            self.started = Some(Instant::now());
        }
        released
    }

    pub fn stop(&mut self) -> Vec<KeyEvent> {
        let released = self.release_all();
        self.status = Status::Stopped;
        self.started = None;
        self.elapsed_before = Duration::ZERO;
        self.delay_until = None;
        self.next_note = 0;
        released
    }

    pub fn pause(&mut self) -> Vec<KeyEvent> {
        if self.status == Status::Playing {
            self.elapsed_before = self.elapsed();
            self.started = None;
            self.status = Status::Paused;
        } else if self.status == Status::Armed {
            self.delay_until = None;
            self.status = Status::Paused;
        }
        self.release_all()
    }

    pub fn resume(&mut self, delay: Duration) {
        if self.status != Status::Paused {
            return;
        }
        if delay.is_zero() {
            self.delay_until = None;
            self.started = Some(Instant::now());
            self.status = Status::Playing;
        } else {
            self.delay_until = Some(Instant::now() + delay);
            self.status = Status::Armed;
        }
    }

    pub fn elapsed(&self) -> Duration {
        match self.started {
            Some(instant) => self.elapsed_before.saturating_add(instant.elapsed()),
            None => self.elapsed_before,
        }
    }

    pub fn position_ms(&self, song: &Song) -> u64 {
        ((self.elapsed().as_secs_f64() * self.speed as f64 * 1000.0) as u64)
            .min(song.duration_ms)
    }

    
    
    pub fn seek(&mut self, song: &Song, target_ms: u64) -> Vec<KeyEvent> {
        let releases = self.release_all();
        let clamped = target_ms.min(song.duration_ms);
        self.next_note = song.notes.partition_point(|n| n.time_ms < clamped);
        self.elapsed_before = Duration::from_secs_f64(
            clamped as f64 / (self.speed.max(0.1) as f64 * 1000.0)
        );
        if self.status == Status::Playing {
            self.started = Some(Instant::now());
        } else {
            self.started = None;
        }
        releases
    }

    pub fn countdown(&self) -> u64 {
        self.delay_until
            .and_then(|end| end.checked_duration_since(Instant::now()))
            .map(|remaining| remaining.as_secs().saturating_add(1))
            .unwrap_or(0)
    }

    pub fn tick(&mut self, song: &Song) -> Vec<KeyEvent> {
        if self.status == Status::Armed {
            if self.delay_until.is_some_and(|end| Instant::now() >= end) {
                self.started = Some(Instant::now());
                self.delay_until = None;
                self.status = Status::Playing;
            } else {
                return Vec::new();
            }
        }
        if self.status != Status::Playing {
            return Vec::new();
        }
        let pos = self.position_ms(song);
        let mut actions = Vec::new();

        for (key, active) in self.active_until.iter_mut().enumerate() {
            if active.is_some_and(|until| until <= pos) {
                *active = None;
                actions.push(KeyEvent { key: key as u8, down: false });
            }
        }

        while self.next_note < song.notes.len() && song.notes[self.next_note].time_ms <= pos {
            let note = &song.notes[self.next_note];
            let idx = note.key as usize;
            
            if self.active_until[idx].is_some() {
                actions.push(KeyEvent { key: note.key, down: false });
            }
            actions.push(KeyEvent { key: note.key, down: true });
            self.active_until[idx] = Some(note.time_ms.saturating_add(note.hold_ms).max(pos + 16));
            self.next_note += 1;
        }
        if pos >= song.duration_ms && self.next_note >= song.notes.len() {
            actions.extend(self.release_all());
            self.status = Status::Finished;
            self.started = None;
            self.elapsed_before = Duration::from_secs_f64(song.duration_ms as f64 / self.speed as f64 / 1000.0);
        }
        actions
    }

    pub fn release_all(&mut self) -> Vec<KeyEvent> {
        let mut result = Vec::new();
        for (key, active) in self.active_until.iter_mut().enumerate() {
            if active.take().is_some() {
                result.push(KeyEvent { key: key as u8, down: false });
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::song::{Note, Song};

    fn sample() -> Song {
        Song { name: "x".into(), notes: vec![
            Note { time_ms: 0, key: 2, hold_ms: 90 },
            Note { time_ms: 0, key: 3, hold_ms: 90 },
        ], duration_ms: 450 }
    }

    #[test]
    fn starts_chord_and_releases_on_stop() {
        let mut player = Player::default();
        player.start(Duration::ZERO);
        assert_eq!(player.tick(&sample()), vec![
            KeyEvent { key: 2, down: true },
            KeyEvent { key: 3, down: true },
        ]);
        assert_eq!(player.stop(), vec![
            KeyEvent { key: 2, down: false },
            KeyEvent { key: 3, down: false },
        ]);
    }

    #[test]
    fn seek_skips_past_events_and_releases_held_keys() {
        let mut player = Player::default();
        player.start(Duration::ZERO);
        player.tick(&sample());
        let released = player.seek(&sample(), 200);
        assert_eq!(released.len(), 2);
        assert_eq!(player.next_note, 2);
        assert!(player.tick(&sample()).is_empty());
        assert!(player.position_ms(&sample()) >= 200);
    }

    #[test]
    fn pause_makes_no_more_events() {
        let mut player = Player::default();
        player.start(Duration::ZERO);
        player.tick(&sample());
        assert_eq!(player.pause().len(), 2);
        assert_eq!(player.status, Status::Paused);
        assert!(player.tick(&sample()).is_empty());
    }
}
