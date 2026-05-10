## Help
help-header = <b>📖 Bantuan Ferris Bot</b>

    Pilih modul di bawah untuk melihat perintah yang tersedia.
    Gunakan tombol untuk navigasi antar modul.

help-admin = <b>👮 Modul Admin</b>

    Mudah untuk mempromosikan dan menurunkan pengguna dengan modul admin!

    <b>Perintah Pengguna:</b>
    - /adminlist: Daftar admin di chat ini.

    <b>Perintah Admin:</b>
    - /promote <code>&lt;reply/username/userid&gt;</code>: Promosikan pengguna.
    - /demote <code>&lt;reply/username/userid&gt;</code>: Turunkan pengguna.
    - /title <code>&lt;reply&gt;</code> <code>&lt;judul&gt;</code>: Atur judul admin (maks 16 karakter).

    <i>Catatan: Anda dan bot memerlukan izin promote/demote.</i>

help-bans = <b>🚫 Modul Ban</b>

    Kadang pengguna bisa mengganggu dan Anda mungkin ingin mengeluarkan mereka!

    <b>Perintah Admin:</b>
    - /ban <code>&lt;user&gt;</code>: Ban pengguna.
    - /sban <code>&lt;user&gt;</code>: Ban diam-diam.
    - /dban <code>&lt;reply&gt;</code>: Ban dan hapus pesan.
    - /tban <code>&lt;user&gt;</code> <code>&lt;waktu&gt;</code>: Ban sementara.
    - /kick <code>&lt;user&gt;</code>: Kick pengguna.
    - /dkick <code>&lt;reply&gt;</code>: Hapus pesan dan kick.
    - /unban <code>&lt;user&gt;</code>: Unban pengguna.

    <i>Balas pesan atau berikan ID/username pengguna.</i>

help-mutes = <b>🔇 Modul Mute</b>

    <b>Perintah Admin:</b>
    - /mute <code>&lt;user&gt;</code>: Bisukan pengguna.
    - /tmute <code>&lt;user&gt;</code> <code>&lt;waktu&gt;</code>: Bisukan sementara.
    - /unmute <code>&lt;user&gt;</code>: Batalkan bisukan.

    <i>Pengguna yang dibisukan tidak bisa mengirim pesan di grup.</i>

help-warns = <b>⚠️ Modul Peringatan</b>

    Jaga anggota tetap tertib dengan peringatan!

    <b>Perintah Admin:</b>
    - /warn <code>&lt;user&gt;</code> <code>[alasan]</code>: Peringatkan pengguna.
    - /warns <code>&lt;user&gt;</code>: Lihat peringatan pengguna.
    - /resetwarns <code>&lt;user&gt;</code>: Reset semua peringatan.
    - /setwarnlimit <code>&lt;angka&gt;</code>: Atur batas peringatan.
    - /setwarnmode <code>&lt;ban/kick/mute&gt;</code>: Atur tindakan saat melebihi batas.

help-notes = <b>📝 Modul Catatan</b>

    Simpan data untuk pengguna di masa depan dengan catatan!

    <b>Perintah Pengguna:</b>
    - /get <code>&lt;nama&gt;</code>: Ambil catatan.
    - <code>#nama</code>: Sama seperti /get.
    - /notes: Daftar semua catatan di chat ini.

    <b>Perintah Admin:</b>
    - /save <code>&lt;nama&gt;</code> <code>&lt;teks&gt;</code>: Simpan catatan baru.
    - /clear <code>&lt;nama&gt;</code>: Hapus catatan.

help-filters = <b>🔍 Modul Filter</b>

    Filter tidak peka huruf besar/kecil; setiap kali seseorang mengatakan kata pemicu, bot akan membalas!

    <b>Perintah:</b>
    - /filter <code>&lt;kata kunci&gt;</code> <code>&lt;balasan&gt;</code>: Tambah filter auto-reply.
    - /filters: Daftar semua filter.
    - /stop <code>&lt;kata kunci&gt;</code>: Hapus filter.
    - /stopall: Hapus SEMUA filter.

help-welcome = <b>👋 Modul Selamat Datang/Selamat Tinggal</b>

    Sambut anggota baru atau ucapkan selamat tinggal saat mereka pergi!

    <b>Perintah Admin:</b>
    - /setwelcome <code>&lt;teks&gt;</code>: Atur teks selamat datang.
    - /welcome <code>&lt;on/off&gt;</code>: Aktifkan/nonaktifkan pesan selamat datang.
    - /setgoodbye <code>&lt;teks&gt;</code>: Atur teks selamat tinggal.
    - /goodbye <code>&lt;on/off&gt;</code>: Aktifkan/nonaktifkan pesan selamat tinggal.
    - /cleanservice <code>&lt;on/off&gt;</code>: Hapus notifikasi bergabung/keluar.

help-rules = <b>📏 Modul Aturan</b>

    Setiap chat punya aturan berbeda; modul ini membantu memperjelas aturan!

    <b>Perintah:</b>
    - /rules: Lihat aturan chat.
    - /setrules <code>&lt;teks&gt;</code>: Atur aturan chat.
    - /clearrules: Hapus aturan chat.

help-blacklist = <b>🚫 Modul Daftar Hitam</b>

    <b>Perintah:</b>
    - /blacklist: Lihat kata yang masuk daftar hitam.
    - /addblacklist <code>&lt;kata&gt;</code>: Tambah ke daftar hitam.
    - /rmblacklist <code>&lt;kata&gt;</code>: Hapus dari daftar hitam.
    - /blacklistmode <code>&lt;delete/warn/mute/kick/ban&gt;</code>: Atur tindakan.

help-purges = <b>🧹 Modul Purge</b>

    <b>Perintah Admin:</b>
    - /purge: Hapus semua pesan antara ini dan pesan yang dibalas.
    - /del: Hapus pesan yang dibalas.

    <i>Bot memerlukan izin "Hapus Pesan".</i>

help-pins = <b>📌 Modul Pin</b>

    <b>Perintah Admin:</b>
    - /pin: Pin pesan yang dibalas.
    - /unpin: Lepas pin pesan saat ini.
    - /unpinall: Lepas semua pin.

    <i>Bot memerlukan izin "Pin Pesan".</i>

help-antiflood = <b>🌊 Modul Antiflood</b>

    Cegah flooding di chat Anda!

    <b>Perintah Admin:</b>
    - /flood: Lihat pengaturan antiflood.
    - /setflood <code>&lt;angka/off&gt;</code>: Atur batas pesan.
    - /setfloodmode <code>&lt;ban/kick/mute&gt;</code>: Pilih tindakan untuk pelaku flood.

help-disable = <b>🔒 Modul Nonaktifkan</b>

    Nonaktifkan perintah umum di grup Anda.

    <b>Perintah Admin:</b>
    - /disable <code>&lt;perintah&gt;</code>: Nonaktifkan perintah.
    - /enable <code>&lt;perintah&gt;</code>: Aktifkan kembali perintah.
    - /disabled: Daftar perintah yang dinonaktifkan.

help-locks = <b>🔐 Modul Kunci</b>

    <b>Perintah Admin:</b>
    - /lock <code>&lt;tipe&gt;</code>: Kunci izin chat.
    - /unlock <code>&lt;tipe&gt;</code>: Buka kunci izin chat.
    - /locks: Lihat kunci saat ini.
    - /locktypes: Lihat tipe kunci yang tersedia.

help-logchannel = <b>📋 Modul Log Channel</b>

    <b>Perintah Admin:</b>
    - /logchannel: Info log channel.
    - /setlogchannel <code>&lt;channel_id&gt;</code>: Atur log channel.
    - /unsetlogchannel: Hapus log channel.

help-reports = <b>📢 Modul Laporan</b>

    Biarkan anggota Anda membantu moderasi!

    <b>Perintah:</b>
    - /report <code>&lt;alasan&gt;</code>: Balas pesan untuk melaporkan ke admin.
    - @admin: Sama seperti /report.
    - /reports <code>&lt;on/off&gt;</code>: Aktifkan/nonaktifkan pelaporan.

help-gbans = <b>🌐 Modul Ban Global</b>

    <b>Perintah Owner/Sudo:</b>
    - /gban <code>&lt;user&gt;</code> <code>[alasan]</code>: Ban global pengguna.
    - /ungban <code>&lt;user&gt;</code>: Hapus ban global.
    - /gbanlist: Daftar semua ban global.

help-backups = <b>💾 Modul Cadangan</b>

    <b>Perintah Owner:</b>
    - /export: Ekspor pengaturan chat sebagai JSON.
    - /import: Impor pengaturan dari file cadangan.

help-connections = <b>🔗 Modul Koneksi</b>

    Hubungkan ke database chat dan kelola dari jarak jauh!

    <b>Perintah:</b>
    - /connect <code>&lt;chatid&gt;</code>: Hubungkan ke chat.
    - /disconnect: Putuskan koneksi.
    - /connection: Info koneksi saat ini.

help-afk = <b>💤 Modul AFK</b>

    <b>Perintah:</b>
    - /afk <code>[alasan]</code>: Tandai diri Anda sebagai AFK.
    - Mengirim <code>brb</code> juga mengatur status AFK.
    - Mengirim pesan apapun menghapus status AFK.

help-blstickers = <b>🎨 Modul Blacklist Stiker</b>

    <b>Perintah Admin:</b>
    - /blsticker: Lihat set stiker yang diblokir.
    - /addblsticker <code>&lt;nama_set&gt;</code>: Blokir set stiker.
    - /rmblsticker <code>&lt;nama_set&gt;</code>: Hapus dari blacklist.
    - /blstickermode <code>&lt;off/del/warn/mute/kick/ban&gt;</code>: Atur tindakan.

help-chatperms = <b>🛡️ Modul Izin Chat</b>

    <b>Perintah Admin:</b>
    - /permissions: Lihat izin chat saat ini.
    - /setpermissions <code>key=on/off</code>: Atur izin.

help-users = <b>👥 Modul Pengguna</b>

    Pelacak pengguna dan chat otomatis.

    <b>Perintah Tim:</b>
    - /stats: Tampilkan statistik bot.
    - /chatlist: Daftar semua chat aktif.

help-misc = <b>📊 Modul Lain-lain</b>

    <b>Perintah:</b>
    - /id: Dapatkan ID grup.
    - /info <code>&lt;user&gt;</code>: Info pengguna.
    - /setlang <code>&lt;en/id&gt;</code>: Atur bahasa bot.
    - /settings: Buka panel pengaturan.

help-captcha = <b>🔐 Modul Captcha</b>

    Lindungi grup dari bot dengan verifikasi CAPTCHA!

    <b>Perintah Admin:</b>
    - /captcha <code>&lt;on/off&gt;</code>: Aktifkan/nonaktifkan captcha.
    - /captchamode <code>&lt;math/text&gt;</code>: Atur tipe captcha.
    - /captchatime <code>&lt;1-10&gt;</code>: Atur batas waktu (menit).
    - /captchaaction <code>&lt;kick/ban/mute&gt;</code>: Atur tindakan gagal.

help-devs = <b>🛠 Modul Developer</b>

    Perintah manajemen bot untuk owner dan developer.

    <b>Perintah:</b>
    - /addsudo, /remsudo: Kelola pengguna sudo.
    - /adddev, /remdev: Kelola developer.
    - /teamusers: Daftar anggota tim.
    - /broadcast <code>&lt;teks&gt;</code>: Broadcast ke semua chat.
    - /botstats: Lihat statistik detail.

help-feds = <b>🏛 Modul Federasi</b>

    Manajemen grup federasi — ban pengguna di semua grup Anda!

    <b>Perintah:</b>
    - /newfed <code>&lt;nama&gt;</code>: Buat federasi.
    - /delfed: Hapus federasi.
    - /joinfed <code>&lt;fed_id&gt;</code>: Gabung federasi.
    - /leavefed: Keluar dari federasi.
    - /fban <code>&lt;user&gt;</code>: Ban federasi.
    - /unfban <code>&lt;user&gt;</code>: Hapus ban federasi.
    - /fbanlist: Daftar ban federasi.

help-sed = <b>✏️ Modul Sed/Regex</b>

    <b>Penggunaan:</b>
    Balas pesan dengan:
    <code>s/pola/pengganti/flag</code>

    <b>Flag:</b> <b>i</b> — Tidak peka huruf, <b>g</b> — Ganti semua

help-userinfo = <b>📋 Modul Bio &amp; Info</b>

    <b>Perintah:</b>
    - /setbio <code>&lt;teks&gt;</code>: Atur bio pengguna lain (balas).
    - /bio <code>[user]</code>: Lihat bio pengguna.
    - /setme <code>&lt;teks&gt;</code>: Atur info Anda.
    - /me <code>[user]</code>: Lihat info pengguna.

help-cleaner = <b>🧹 Modul Pembersih</b>

    <b>Perintah Admin:</b>
    - /cleanservice <code>&lt;on/off&gt;</code>: Hapus otomatis pesan layanan.
    - /cleanbluetext <code>&lt;on/off&gt;</code>: Hapus otomatis perintah bot tak dikenal.

help-reactions = <b>⚡ Modul Reaksi</b>

    <b>Perintah Admin:</b>
    - /addreaction <code>&lt;kata kunci&gt;</code> <code>&lt;emoji&gt;</code>: Reaksi saat kata kunci disebut.
    - /removereaction <code>&lt;kata kunci&gt;</code>: Hapus reaksi.
    - /reactions: Daftar semua reaksi.
    - /resetreactions: Hapus semua reaksi.

help-about = <b>ℹ️ Tentang Ferris Bot</b>

    🦀 Dibuat dengan Rust menggunakan Teloxide v0.17.0
    📦 Database PostgreSQL dengan sqlx
    ⚡ Redis caching &amp; performa async

    <b>Fitur:</b>
    • Manajemen grup lengkap (30+ modul)
    • Dukungan multi-bahasa (EN/ID)
    • Sistem federasi
    • Arsitektur modular

    Dibuat dengan ❤️ oleh Arumi

## Formatting
help-formatting = <b>📝 Pemformatan</b>

    Ferris Bot mendukung banyak opsi pemformatan untuk membuat pesan Anda lebih ekspresif. Lihat dengan mengklik tombol di bawah!

help-formatting-markdown = <b>Pemformatan Markdown</b>

    Anda bisa memformat pesan menggunakan <b>tebal</b>, <i>miring</i>, <u>garis bawah</u>, dan lainnya. Silakan bereksperimen!

    <b>Markdown yang didukung:</b>
    - <code>`teks kode`</code>: Backtick untuk monospace. Tampil sebagai: <code>teks kode</code>.
    - <code>_teks miring_</code>: Garis bawah untuk miring. Tampil sebagai: <i>teks miring</i>.
    - <code>*teks tebal*</code>: Asterisk untuk tebal. Tampil sebagai: <b>teks tebal</b>.
    - <code>~coret~</code>: Tilde untuk coret. Tampil sebagai: <s>coret</s>.
    - <code>||spoiler||</code>: Dua garis vertikal untuk spoiler. Tampil sebagai: <tg-spoiler>Spoiler</tg-spoiler>.
    - <code>```pre```</code>: Tiga backtick untuk teks preformatted.
    - <code>__garis bawah__</code>: Dua garis bawah untuk underline.
    - <code>[hyperlink](example.com)</code>: Membuat hyperlink. Tampil sebagai: <a href="https://example.com/">hyperlink</a>.
    - <code>[Tombol Saya](buttonurl://example.com)</code>: Membuat tombol bernama "Tombol Saya" yang membuka <code>example.com</code>.

    Untuk menempatkan tombol di baris yang sama, gunakan <code>:same</code>:
    <code>[tombol 1](buttonurl://example.com)</code>
    <code>[tombol 2](buttonurl://example.com:same)</code>

help-formatting-fillings = <b>Pengisian</b>

    Anda bisa menyesuaikan isi pesan dengan data kontekstual. Misalnya, sebut pengguna berdasarkan nama di pesan selamat datang, atau sebut mereka di filter!

    <b>Pengisian yang didukung:</b>
    - <code>{first_name}</code>: Nama depan pengguna.
    - <code>{last_name}</code>: Nama belakang pengguna.
    - <code>{full_name}</code>: Nama lengkap pengguna.
    - <code>{username}</code>: Username pengguna. Jika tidak ada, menyebut pengguna.
    - <code>{mention}</code>: Menyebut pengguna dengan nama depan.
    - <code>{id}</code>: ID pengguna.
    - <code>{chat_name}</code>: Nama chat.

## Contoh penggunaan per modul
help-example-admin = <b>💡 Contoh Penggunaan — Admin</b>

    <code>/promote @username</code>
    <code>/demote @username</code>
    <code>/title Judul Baru</code> (balas ke pengguna)
    <code>/adminlist</code>

help-example-bans = <b>💡 Contoh Penggunaan — Ban</b>

    <code>/ban @spammer Spam tautan</code>
    <code>/tban @user 2h Waktu istirahat</code>
    <code>/dban</code> (balas pesan yang melanggar)
    <code>/kick @user</code>
    <code>/unban @user</code>

help-example-mutes = <b>💡 Contoh Penggunaan — Mute</b>

    <code>/mute @user</code>
    <code>/tmute @user 30m</code>
    <code>/unmute @user</code>

help-example-warns = <b>💡 Contoh Penggunaan — Peringatan</b>

    <code>/warn @user Melanggar aturan</code>
    <code>/setwarnlimit 5</code>
    <code>/setwarnmode mute</code>
    <code>/warns @user</code>
    <code>/resetwarns @user</code>

help-example-notes = <b>💡 Contoh Penggunaan — Catatan</b>

    <code>/save aturan Silakan ikuti aturan grup!</code>
    <code>/get aturan</code>
    <code>#aturan</code>
    <code>/notes</code>
    <code>/clear aturan</code>

help-example-filters = <b>💡 Contoh Penggunaan — Filter</b>

    <code>/filter halo Halo! Apa kabar?</code>
    <code>/filter "halo teman" Halo juga!</code>
    <code>/stop halo</code>
    <code>/filters</code>

    <i>Untuk menyimpan file/gambar/gif, balas file dengan:</i>
    <code>/filter pemicu</code>

help-example-welcome = <b>💡 Contoh Penggunaan — Selamat Datang</b>

    <code>/setwelcome Hai {first_name}, selamat datang di {chat_name}! Baca /rules dulu.</code>
    <code>/welcome on</code>
    <code>/setgoodbye Selamat tinggal {first_name}, kami akan merindukanmu!</code>
    <code>/goodbye on</code>
    <code>/cleanservice on</code>

help-example-rules = <b>💡 Contoh Penggunaan — Aturan</b>

    <code>/setrules 1. Dilarang spam
    2. Hormati sesama
    3. Bahasa Indonesia saja</code>
    <code>/rules</code>
    <code>/clearrules</code>

help-example-blacklist = <b>💡 Contoh Penggunaan — Daftar Hitam</b>

    <code>/addblacklist spam</code>
    <code>/blacklistmode kick</code>
    <code>/blacklist</code>
    <code>/rmblacklist spam</code>

help-example-purges = <b>💡 Contoh Penggunaan — Purge</b>

    → Balas pesan, lalu kirim <code>/purge</code>
    → <code>/del</code> (balas pesan yang melanggar)

help-example-pins = <b>💡 Contoh Penggunaan — Pin</b>

    <code>/pin</code> (balas pesan)
    <code>/pin loud</code> (pin dengan notifikasi)
    <code>/unpin</code>
    <code>/unpinall</code>

help-example-antiflood = <b>💡 Contoh Penggunaan — Antiflood</b>

    <code>/setflood 10</code>
    <code>/setfloodmode mute</code>
    <code>/setflood off</code>
    <code>/flood</code>

help-example-disable = <b>💡 Contoh Penggunaan — Nonaktifkan</b>

    <code>/disable rules</code>
    <code>/enable rules</code>
    <code>/disabled</code>

    <i>Perintah yang dinonaktifkan hanya berlaku untuk non-admin.</i>

help-example-locks = <b>💡 Contoh Penggunaan — Kunci</b>

    <code>/lock media</code> — kunci semua media
    <code>/lock url</code> — hapus otomatis URL
    <code>/unlock all</code> — hapus semua kunci
    <code>/locks</code>
    <code>/locktypes</code>

help-example-logchannel = <b>💡 Contoh Penggunaan — Log Channel</b>

    1. Tambahkan bot ke channel (sebagai admin)
    2. <code>/setlogchannel -1001234567890</code>
    3. <code>/logchannel</code>
    4. <code>/unsetlogchannel</code>

help-example-reports = <b>💡 Contoh Penggunaan — Laporan</b>

    → Balas pesan spam: <code>/report Spam</code>
    → Atau tag <code>@admin</code> di grup
    <code>/reports on</code>

help-example-gbans = <b>💡 Contoh Penggunaan — Ban Global</b>

    <code>/gban @spammer Spam di banyak grup</code>
    <code>/ungban 123456789</code>
    <code>/gbanlist</code>

help-example-backups = <b>💡 Contoh Penggunaan — Cadangan</b>

    <code>/export</code> — ekspor pengaturan chat sebagai JSON
    <code>/import</code> — balas file cadangan untuk mengimpor

help-example-connections = <b>💡 Contoh Penggunaan — Koneksi</b>

    <code>/connect -1001234567890</code>
    <code>/disconnect</code>
    <code>/connection</code>

help-example-afk = <b>💡 Contoh Penggunaan — AFK</b>

    <code>/afk Pergi makan siang</code>
    <code>/afk</code>
    Cukup ketik <code>brb</code> untuk AFK.

help-example-blstickers = <b>💡 Contoh Penggunaan — Blacklist Stiker</b>

    <code>/addblsticker</code> (balas ke stiker)
    <code>/blstickermode ban</code>
    <code>/blsticker</code>
    <code>/rmblsticker nama_set</code>

help-example-chatperms = <b>💡 Contoh Penggunaan — Izin Chat</b>

    <code>/setpermissions stickers=off polls=off</code>
    <code>/setpermissions media=on</code>
    <code>/permissions</code>

help-example-users = <b>💡 Contoh Penggunaan — Pengguna</b>

    <code>/stats</code>
    <code>/chatlist</code>

help-example-misc = <b>💡 Contoh Penggunaan — Lain-lain</b>

    <code>/info @username</code>
    <code>/id</code> (balas pesan)
    <code>/setlang id</code>
    <code>/settings</code>

help-example-captcha = <b>💡 Contoh Penggunaan — Captcha</b>

    <code>/captcha on</code>
    <code>/captchamode math</code>
    <code>/captchatime 5</code>
    <code>/captchaaction kick</code>
    <code>/captchaattempts 3</code>

help-example-devs = <b>💡 Contoh Penggunaan — Developer</b>

    <code>/addsudo @trusted_user</code>
    <code>/broadcast Halo semuanya!</code>
    <code>/leavechat -1001234567890</code>
    <code>/botstats</code>
    <code>/teamusers</code>

help-example-feds = <b>💡 Contoh Penggunaan — Federasi</b>

    <code>/newfed Jaringan Saya</code>
    <code>/joinfed fed_id_disini</code>
    <code>/fban @spammer Spam di semua grup</code>
    <code>/unfban @user</code>
    <code>/fbanlist</code>
    <code>/fedrules Hormati semua chat federasi</code>

help-example-sed = <b>💡 Contoh Penggunaan — Sed/Regex</b>

    Balas pesan dengan:
    <code>s/halo/selamat tinggal/gi</code>
    <code>s|typo|diperbaiki|</code>
    <code>s/teh/the/g</code>

help-example-userinfo = <b>💡 Contoh Penggunaan — Bio &amp; Info</b>

    <code>/setme Penggemar Rust dan pengembang bot</code>
    <code>/setbio Orang ini keren!</code> (balas ke pengguna)
    <code>/bio @username</code>
    <code>/me</code>

help-example-cleaner = <b>💡 Contoh Penggunaan — Pembersih</b>

    <code>/cleanservice on</code>
    <code>/cleanbluetext on</code>

help-example-reactions = <b>💡 Contoh Penggunaan — Reaksi</b>

    <code>/addreaction halo 👋</code>
    <code>/addreaction rust 🦀</code>
    <code>/removereaction halo</code>
    <code>/reactions</code>
    <code>/resetreactions</code>
