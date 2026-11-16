

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Language {
    #[default]
    Ru, En, Ja, ZhCn, ZhTw, Ms, PtBr, Id, Ko, Fil,
}
impl Language {
    pub const ALL: [Language; 10] = [Language::Ru, Language::En, Language::Ja, Language::ZhCn, Language::ZhTw, Language::Ms, Language::PtBr, Language::Id, Language::Ko, Language::Fil];
    pub const fn index(self) -> usize { self as usize }
    pub const fn code(self) -> &'static str { ["ru", "en", "ja", "zh-CN", "zh-TW", "ms", "pt-BR", "id", "ko", "fil"][self.index()] }
    pub fn from_code(value: &str) -> Option<Self> { Self::ALL.into_iter().find(|lang| lang.code() == value) }
    pub const fn native_name(self) -> &'static str { ["Русский", "English", "日本語", "简体中文", "繁體中文（台灣）", "Bahasa Melayu", "Português (Brasil)", "Bahasa Indonesia", "한국어", "Filipino"][self.index()] }
    pub fn shifted(self, offset: i32) -> Self {
        Self::ALL[(self.index() as i32 + offset).rem_euclid(Self::ALL.len() as i32) as usize]
    }
    pub fn tr(self, key: &'static str) -> &'static str {
        match key {
            "library" => ["Библиотека", "Library", "ライブラリ", "曲库", "曲庫", "Pustaka", "Biblioteca", "Pustaka", "라이브러리", "Aklatan"][self.index()],
            "favorites_all" => ["Все", "All", "すべて", "全部", "全部", "Semua", "Todas", "Semua", "전체", "Lahat"][self.index()],
            "favorites_only" => ["Избранное", "Favorites", "お気に入り", "收藏", "收藏", "Kegemaran", "Favoritas", "Favorit", "즐겨찾기", "Paborito"][self.index()],
            "favorites_empty" => ["В избранном пока нет песен", "No favorite songs yet", "お気に入りの曲はまだありません", "暂无收藏歌曲", "尚無收藏歌曲", "Tiada lagu kegemaran lagi", "Ainda não há músicas favoritas", "Belum ada lagu favorit", "즐겨찾는 곡이 없습니다", "Wala pang paboritong kanta"][self.index()],
            "import_song" => ["Импортировать композицию", "Import a song", "曲をインポート", "导入曲目", "匯入曲目", "Import lagu", "Importar música", "Impor lagu", "곡 가져오기", "Mag-import ng kanta"][self.index()],
            "sort_library" => ["Сортировать по названию", "Sort by title", "名前順に並べ替え", "按名称排序", "依名稱排序", "Isih mengikut tajuk", "Ordenar por nome", "Urutkan berdasarkan judul", "제목순 정렬", "Ayusin ayon sa pamagat"][self.index()],
            "search" => ["Поиск", "Search", "検索", "搜索", "搜尋", "Cari", "Pesquisar", "Cari", "검색", "Maghanap"][self.index()],
            "no_results" => ["Ничего не найдено", "No results", "見つかりません", "没有搜索结果", "找不到結果", "Tiada hasil", "Nenhum resultado", "Tidak ada hasil", "검색 결과 없음", "Walang resulta"][self.index()],
            "rename" => ["Переименовать", "Rename", "名前を変更", "重命名", "重新命名", "Namakan semula", "Renomear", "Ubah nama", "이름 변경", "Palitan ang pangalan"][self.index()],
            "delete" => ["Удалить", "Delete", "削除", "删除", "刪除", "Padam", "Excluir", "Hapus", "삭제", "Burahin"][self.index()],
            "delete_song" => ["Удалить композицию", "Delete song", "曲を削除", "删除曲目", "刪除曲目", "Padam lagu", "Excluir música", "Hapus lagu", "곡 삭제", "Burahin ang kanta"][self.index()],
            "refresh_library" => ["Обновить список песен", "Refresh song list", "曲一覧を更新", "刷新曲目列表", "重新整理曲目清單", "Muat semula senarai lagu", "Atualizar lista de músicas", "Segarkan daftar lagu", "곡 목록 새로고침", "I-refresh ang listahan"][self.index()],
            "settings" => ["Настройки", "Settings", "設定", "设置", "設定", "Tetapan", "Configurações", "Pengaturan", "설정", "Mga setting"][self.index()],
            "updates" => ["Обновления", "Updates", "アップデート", "更新", "更新", "Kemas kini", "Atualizações", "Pembaruan", "업데이트", "Mga update"][self.index()],
            "updates_check" => ["Проверить", "Check now", "確認", "检查", "檢查", "Semak", "Verificar", "Periksa", "확인", "Suriin"][self.index()],
            "updates_checking" => ["Проверка…", "Checking…", "確認中…", "检查中…", "檢查中…", "Menyemak…", "Verificando…", "Memeriksa…", "확인 중…", "Sinusuri…"][self.index()],
            "updates_idle" => ["Нажмите, чтобы проверить обновления", "Check for new releases", "新しいリリースを確認", "检查新版本", "檢查新版本", "Semak keluaran baharu", "Verifique novas versões", "Periksa rilis baru", "새 릴리스 확인", "Suriin ang bagong release"][self.index()],
            "updates_current" => ["Установлена последняя версия", "You have the latest version", "最新版です", "已是最新版本", "已是最新版本", "Versi terkini dipasang", "Você tem a versão mais recente", "Versi sudah terbaru", "최신 버전입니다", "Pinakabagong bersyon na"][self.index()],
            "updates_available" => ["Доступна новая версия", "New version available", "新しいバージョン", "有新版本", "有新版本", "Versi baharu tersedia", "Nova versão disponível", "Versi baru tersedia", "새 버전 제공", "May bagong bersyon"][self.index()],
            "updates_error" => ["Не удалось проверить обновления", "Could not check for updates", "更新の確認に失敗", "无法检查更新", "無法檢查更新", "Semakan kemas kini gagal", "Falha ao verificar", "Gagal memeriksa pembaruan", "업데이트 확인 실패", "Hindi masuri ang update"][self.index()],
            "updates_open_release" => ["Открыть GitHub Release ↗", "Open GitHub Release ↗", "GitHub Release を開く ↗", "打开 GitHub Release ↗", "開啟 GitHub Release ↗", "Buka GitHub Release ↗", "Abrir GitHub Release ↗", "Buka GitHub Release ↗", "GitHub Release 열기 ↗", "Buksan ang GitHub Release ↗"][self.index()],
            "song_one" => ["песня", "song", "曲", "首曲目", "首曲目", "lagu", "música", "lagu", "곡", "kanta"][self.index()],
            "song_few" => ["песни", "songs", "曲", "首曲目", "首曲目", "lagu", "músicas", "lagu", "곡", "kanta"][self.index()],
            "song_many" => ["песен", "songs", "曲", "首曲目", "首曲目", "lagu", "músicas", "lagu", "곡", "kanta"][self.index()],
            "delete_question" => ["Удалить композицию?", "Delete this song?", "この曲を削除しますか？", "删除这首曲目？", "刪除這首曲目？", "Padam lagu ini?", "Excluir esta música?", "Hapus lagu ini?", "이 곡을 삭제할까요?", "Burahin ang kantang ito?"][self.index()],
            "delete_explainer" => ["Удалить композицию из библиотеки?", "Remove this song from the library?", "ライブラリから削除しますか？", "从曲库中删除这首曲目？", "要從曲庫刪除這首曲目嗎？", "Alih keluar lagu ini daripada pustaka?", "Remover esta música da biblioteca?", "Hapus lagu ini dari pustaka?", "라이브러리에서 삭제할까요?", "Alisin ang kanta sa aklatan?"][self.index()],
            "delete_file_prefix" => ["Файл будет удалён с диска:", "The file will be deleted from disk:", "ファイルもディスクから削除されます：", "文件也将从磁盘删除：", "檔案也將從磁碟刪除：", "Fail juga akan dipadam daripada cakera:", "O arquivo também será excluído do disco:", "File juga akan dihapus dari disk:", "파일도 디스크에서 삭제됩니다:", "Mabubura rin ang file sa disk:"][self.index()],
            "cancel" => ["Отмена", "Cancel", "キャンセル", "取消", "取消", "Batal", "Cancelar", "Batal", "취소", "Kanselahin"][self.index()],
            "dark_theme" => ["Тёмная тема", "Dark theme", "ダークテーマ", "深色主题", "深色主題", "Tema gelap", "Tema escuro", "Tema gelap", "다크 테마", "Madilim na tema"][self.index()],
            "both_windows" => ["Одновременно для обоих окон", "Applies to both windows", "両方のウィンドウに適用", "同时应用于两个窗口", "同時套用至兩個視窗", "Digunakan pada kedua-dua tetingkap", "Aplica-se às duas janelas", "Berlaku pada kedua jendela", "두 창 모두에 적용", "Para sa parehong window"][self.index()],
            "sky_delay" => ["Задержка перед Sky", "Delay before Sky", "Sky開始までの待機", "发送到 Sky 前的延迟", "傳送到 Sky 前的延遲", "Jeda sebelum Sky", "Atraso antes do Sky", "Jeda sebelum Sky", "Sky 시작 전 대기", "Antala bago ang Sky"][self.index()],
            "seconds" => ["с", "s", "秒", "秒", "秒", "s", "s", "d", "초", "seg"][self.index()],
            "delay_explainer" => ["От 0 до 10 секунд для переключения в игру", "0–10 seconds to switch to the game", "ゲームに切り替えるまで0～10秒", "0–10 秒内切换至游戏", "0–10 秒內切換至遊戲", "0–10 saat untuk beralih ke permainan", "0–10 segundos para mudar para o jogo", "0–10 detik untuk beralih ke game", "게임으로 전환할 시간: 0~10초", "0–10 segundo para lumipat sa laro"][self.index()],
            "auto_next" => ["Играть следующую песню", "Play next song automatically", "次の曲を自動再生", "自动播放下一首", "自動播放下一首", "Main lagu seterusnya secara automatik", "Reproduzir próxima música automaticamente", "Putar lagu berikutnya otomatis", "다음 곡 자동 재생", "Awtomatikong patugtugin ang susunod"][self.index()],
            "auto_save" => ["Настройки сохраняются автоматически", "Settings are saved automatically", "設定は自動保存されます", "设置将自动保存", "設定會自動儲存", "Tetapan disimpan secara automatik", "Configurações salvas automaticamente", "Pengaturan disimpan otomatis", "설정이 자동으로 저장됩니다", "Awtomatikong sine-save ang mga setting"][self.index()],
            "language" => ["Язык", "Language", "言語", "语言", "語言", "Bahasa", "Idioma", "Bahasa", "언어", "Wika"][self.index()],
            "previous_language" => ["Предыдущий язык", "Previous language", "前の言語", "上一种语言", "上一種語言", "Bahasa sebelumnya", "Idioma anterior", "Bahasa sebelumnya", "이전 언어", "Nakaraang wika"][self.index()],
            "next_language" => ["Следующий язык", "Next language", "次の言語", "下一种语言", "下一種語言", "Bahasa seterusnya", "Próximo idioma", "Bahasa berikutnya", "다음 언어", "Susunod na wika"][self.index()],
            "rename_song" => ["Переименовать песню", "Rename song", "曲名を変更", "重命名曲目", "重新命名曲目", "Namakan semula lagu", "Renomear música", "Ubah nama lagu", "곡 이름 변경", "Palitan ang pangalan ng kanta"][self.index()],
            "rename_helper" => ["Название в библиотеке и плеерах", "Name shown in the library and players", "ライブラリとプレーヤーに表示する名前", "在曲库和播放器中显示的名称", "在曲庫與播放器顯示的名稱", "Nama dalam pustaka dan pemain", "Nome na biblioteca e nos players", "Nama di pustaka dan pemutar", "라이브러리와 플레이어에 표시할 이름", "Pangalan sa aklatan at mga player"][self.index()],
            "original_preserved" => ["Исходный музыкальный файл останется без изменений", "The original song file will not change", "元の曲ファイルは変更されません", "原始曲目文件不会更改", "原始曲目檔案不會變更", "Fail lagu asal tidak akan diubah", "O arquivo de música original não será alterado", "File lagu asli tidak akan diubah", "원본 곡 파일은 변경되지 않습니다", "Hindi mababago ang orihinal na file"][self.index()],
            "invalid_title" => ["Введите название длиной 1–80 символов без переносов строк.", "Enter a title of 1–80 characters without line breaks.", "改行なしで1～80文字の名前を入力してください。", "请输入 1–80 个字符且不含换行的名称。", "請輸入 1–80 個字元且不含換行的名稱。", "Masukkan nama 1–80 aksara tanpa baris baharu.", "Digite um nome de 1 a 80 caracteres, sem quebras de linha.", "Masukkan nama 1–80 karakter tanpa baris baru.", "줄 바꿈 없이 1~80자의 이름을 입력하세요.", "Maglagay ng 1–80 character na walang line break."][self.index()],
            "save" => ["Сохранить", "Save", "保存", "保存", "儲存", "Simpan", "Salvar", "Simpan", "저장", "I-save"][self.index()],
            "choose_composition" => ["Выберите композицию", "Select a song", "曲を選択", "选择曲目", "選擇曲目", "Pilih lagu", "Selecione uma música", "Pilih lagu", "곡 선택", "Pumili ng kanta"][self.index()],
            "choose_composition_status" => ["Выберите композицию из вашей библиотеки.", "Select a song from your library.", "ライブラリから曲を選択してください。", "请从您的曲库中选择一首曲目。", "請從您的曲庫選擇一首曲目。", "Pilih lagu daripada pustaka anda.", "Selecione uma música da sua biblioteca.", "Pilih lagu dari pustaka Anda.", "라이브러리에서 곡을 선택하세요.", "Pumili ng kanta mula sa iyong aklatan."][self.index()],
            "choose_from_library" => ["Выберите песню из библиотеки", "Choose a song from the library", "ライブラリから曲を選択", "从曲库中选择一首曲目", "從曲庫選擇一首曲目", "Pilih lagu daripada pustaka", "Escolha uma música da biblioteca", "Pilih lagu dari pustaka", "라이브러리에서 곡을 선택하세요", "Pumili ng kanta sa aklatan"][self.index()],
            "composition" => ["Композиция", "Song", "曲", "曲目", "曲目", "Lagu", "Música", "Lagu", "곡", "Kanta"][self.index()],
            "notes" => ["нот", "notes", "音符", "个音符", "個音符", "nota", "notas", "not", "음표", "nota"][self.index()],
            "prev_song" => ["Предыдущая композиция", "Previous song", "前の曲", "上一首", "上一首", "Lagu sebelumnya", "Música anterior", "Lagu sebelumnya", "이전 곡", "Nakaraang kanta"][self.index()],
            "next_song" => ["Следующая композиция", "Next song", "次の曲", "下一首", "下一首", "Lagu seterusnya", "Próxima música", "Lagu berikutnya", "다음 곡", "Susunod na kanta"][self.index()],
            "pause" => ["Пауза", "Pause", "一時停止", "暂停", "暫停", "Jeda", "Pausar", "Jeda", "일시정지", "I-pause"][self.index()],
            "play" => ["Воспроизвести", "Play", "再生", "播放", "播放", "Main", "Reproduzir", "Putar", "재생", "I-play"][self.index()],
            "stop" => ["Остановить", "Stop", "停止", "停止", "停止", "Henti", "Parar", "Hentikan", "정지", "Ihinto"][self.index()],
            "overlay_player" => ["Оверлей плеер", "Overlay player", "オーバーレイプレーヤー", "悬浮播放器", "浮動播放器", "Pemain tindihan", "Player sobreposto", "Pemutar overlay", "오버레이 플레이어", "Overlay player"][self.index()],
            "send_keys" => ["Отправлять нажатия в Sky", "Send key presses to Sky", "Skyにキー入力を送信", "向 Sky 发送按键", "傳送按鍵至 Sky", "Hantar kekunci ke Sky", "Enviar teclas para o Sky", "Kirim tombol ke Sky", "Sky로 키 입력 보내기", "Magpadala ng mga key sa Sky"][self.index()],
            "harp" => ["Арфа", "Harp", "ハープ", "竖琴", "豎琴", "Harpa", "Harpa", "Harpa", "하프", "Alpa"][self.index()],
            "guitar" => ["Гитара", "Guitar", "ギター", "吉他", "吉他", "Gitar", "Violão", "Gitar", "기타", "Gitara"][self.index()],
            "audio_unavailable" => ["Ошибка звукового предпрослушивания", "Audio preview unavailable", "音声プレビューを利用できません", "无法播放音频预览", "無法播放音訊預覽", "Pratonton audio tidak tersedia", "Prévia de áudio indisponível", "Pratinjau audio tidak tersedia", "오디오 미리보기를 사용할 수 없음", "Hindi available ang audio preview"][self.index()],
            "speed" => ["Скорость", "Speed", "速度", "速度", "速度", "Kelajuan", "Velocidade", "Kecepatan", "속도", "Bilis"][self.index()],
            "speed_reset" => ["Сбросить скорость до 1,00×", "Reset speed to 1.00×", "速度を1.00×に戻す", "将速度重置为 1.00×", "將速度重設為 1.00×", "Set semula kelajuan kepada 1.00×", "Redefinir velocidade para 1,00×", "Atur ulang kecepatan ke 1,00×", "속도를 1.00×로 초기화", "I-reset ang bilis sa 1.00×"][self.index()],
            "no_song" => ["Композиция не выбрана", "No song selected", "曲が未選択です", "未选择曲目", "尚未選擇曲目", "Tiada lagu dipilih", "Nenhuma música selecionada", "Belum ada lagu dipilih", "선택한 곡 없음", "Walang napiling kanta"][self.index()],
            "countdown_prefix" => ["Sky: начало через", "Sky: starting in", "Sky：開始まで", "Sky：将在", "Sky：將在", "Sky: bermula dalam", "Sky: começa em", "Sky: dimulai dalam", "Sky: 시작까지", "Sky: magsisimula sa loob ng"][self.index()],
            "waiting_sky" => ["Ожидание Sky", "Waiting for Sky", "Skyを待機中", "等待 Sky", "等待 Sky", "Menunggu Sky", "Aguardando Sky", "Menunggu Sky", "Sky 대기 중", "Naghihintay sa Sky"][self.index()],
            "playing_sky" => ["Играет в Sky", "Playing in Sky", "Skyで演奏中", "正在 Sky 中演奏", "正在 Sky 中演奏", "Bermain dalam Sky", "Tocando no Sky", "Memainkan di Sky", "Sky에서 연주 중", "Tumutugtog sa Sky"][self.index()],
            "playing_preview" => ["Играет · звуковой предпросмотр", "Playing · audio preview", "再生中・音声プレビュー", "播放中 · 音频预览", "播放中 · 音訊預覽", "Main · pratonton audio", "Reproduzindo · prévia de áudio", "Memutar · pratinjau audio", "재생 중 · 오디오 미리보기", "Tumutugtog · audio preview"][self.index()],
            "paused_focus" => ["Пауза · потерян фокус Sky", "Paused · Sky lost focus", "一時停止・Skyのフォーカスが外れました", "已暂停 · Sky 失去焦点", "已暫停 · Sky 失去焦點", "Jeda · fokus Sky hilang", "Pausado · Sky perdeu o foco", "Jeda · fokus Sky hilang", "일시정지 · Sky 포커스 해제", "Naka-pause · nawala ang focus ng Sky"][self.index()],
            "paused_input" => ["Пауза · ошибка ввода", "Paused · input error", "一時停止・入力エラー", "已暂停 · 输入错误", "已暫停 · 輸入錯誤", "Jeda · ralat input", "Pausado · erro de entrada", "Jeda · kesalahan input", "일시정지 · 입력 오류", "Naka-pause · error sa input"][self.index()],
            "finished" => ["Завершено", "Finished", "完了", "已完成", "已完成", "Selesai", "Concluído", "Selesai", "완료", "Tapos"][self.index()],
            "finished_ready" => ["Завершено · готово к повтору", "Finished · ready to replay", "終了・再生準備完了", "已完成 · 可重新播放", "已完成 · 可重新播放", "Selesai · sedia ulang main", "Concluído · pronto para repetir", "Selesai · siap diputar ulang", "완료 · 다시 재생 가능", "Tapos · puwedeng ulitin"][self.index()],
            "sky_ready" => ["Активирован режим проигрывания музыки в Sky.", "Sky mode · ready", "Skyモード・準備完了", "Sky 模式 · 就绪", "Sky 模式 · 就緒", "Mod Sky · sedia", "Modo Sky · pronto", "Mode Sky · siap", "Sky 모드 · 준비됨", "Sky mode · handa"][self.index()],
            "preview_ready" => ["Активирован режим предпросмотра песни.", "Ready · audio preview", "準備完了・音声プレビュー", "就绪 · 音频预览", "就緒 · 音訊預覽", "Sedia · pratonton audio", "Pronto · prévia de áudio", "Siap · pratinjau audio", "준비됨 · 오디오 미리보기", "Handa · audio preview"][self.index()],
            "file_filter" => ["Музыкальные листы Sky", "Sky music sheets", "Sky楽譜", "Sky 乐谱", "Sky 樂譜", "Lembaran muzik Sky", "Partituras do Sky", "Lembar musik Sky", "Sky 악보", "Mga nota sa Sky"][self.index()],
            "label_saved" => ["Название сохранено в настройках; исходный файл не изменён.", "Title saved; original file unchanged.", "名前を保存しました。元ファイルは変更されません。", "名称已保存，原始文件未更改。", "名稱已儲存，原始檔案未變更。", "Nama disimpan; fail asal tidak berubah.", "Nome salvo; arquivo original inalterado.", "Nama disimpan; file asli tidak berubah.", "이름이 저장되었습니다. 원본 파일은 변경되지 않았습니다.", "Na-save ang pangalan; hindi binago ang file."][self.index()],
            _ => key,
        }
    }
    pub fn song_count(self, n: usize) -> String {
        let key = if self == Language::Ru {
            if n % 10 == 1 && n % 100 != 11 { "song_one" }
            else if (2..=4).contains(&(n % 10)) && !(12..=14).contains(&(n % 100)) { "song_few" }
            else { "song_many" }
        } else if n == 1 { "song_one" } else { "song_many" };
        format!("{n} {}", self.tr(key))
    }
}
#[cfg(test)]
mod tests {
    use super::Language;
    #[test]
    fn each_language_has_every_translation() {
        for language in Language::ALL {
            assert_eq!(Language::from_code(language.code()), Some(language));
            assert!(!language.native_name().is_empty());
            for key in ["library", "favorites_all", "favorites_only", "favorites_empty", "import_song", "sort_library", "search", "no_results", "rename", "delete", "delete_song", "refresh_library", "settings", "updates", "updates_check", "updates_checking", "updates_idle", "updates_current", "updates_available", "updates_error", "updates_open_release", "song_one", "song_few", "song_many", "delete_question", "delete_explainer", "delete_file_prefix", "cancel", "dark_theme", "both_windows", "sky_delay", "seconds", "delay_explainer", "auto_next", "auto_save", "language", "previous_language", "next_language", "rename_song", "rename_helper", "original_preserved", "invalid_title", "save", "choose_composition", "choose_composition_status", "choose_from_library", "composition", "notes", "prev_song", "next_song", "pause", "play", "stop", "overlay_player", "send_keys", "speed", "speed_reset", "no_song", "countdown_prefix", "waiting_sky", "playing_sky", "playing_preview", "paused_focus", "paused_input", "finished", "finished_ready", "sky_ready", "preview_ready", "file_filter", "label_saved"] {
                
                assert!(!language.tr(key).is_empty());
            }
        }
        assert_eq!(Language::Ru.shifted(-1), Language::Fil);
        assert_eq!(Language::Fil.shifted(1), Language::Ru);
        assert_eq!(Language::Ru.song_count(1), "1 песня");
        assert_eq!(Language::Ru.song_count(2), "2 песни");
        assert_eq!(Language::Ru.song_count(5), "5 песен");
        assert_eq!(Language::Ru.song_count(21), "21 песня");
        assert_eq!(Language::Ru.tr("sky_ready"), "Активирован режим проигрывания музыки в Sky.");
        assert_eq!(Language::Ru.tr("preview_ready"), "Активирован режим предпросмотра песни.");
        assert_eq!(Language::Ru.tr("choose_composition_status"), "Выберите композицию из вашей библиотеки.");
    }
}
