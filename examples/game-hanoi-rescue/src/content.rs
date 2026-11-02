//! Every word in the game and every number a designer tunes. Change the story here.

use crate::map::ExitId;

pub const TITLE: &str = "GIẢI CỨU CÚN";
pub const SUBTITLE: &str = "Hà Nội · Hoàn Kiếm";
pub const HERO: &str = "Minh";
pub const PUPPY: &str = "Bơ";

/// One line of a story scene: who speaks (empty for narration) and what they say.
pub type Line = (&'static str, &'static str);

pub const INTRO: &[Line] = &[
    ("", "Hà Nội, cuối thu. Minh, sinh viên năm hai, thuê một phòng trọ nhỏ sau phố cổ."),
    ("", "Bạn thân nhất của cậu là Bơ, một chú chó cỏ lông vàng, tai cụp, ăn khỏe ngủ khỏe."),
    ("", "Tối nay, Minh vừa quay lưng mua bánh mì, một chiếc xe máy rồ ga. Thòng lọng. Tiếng Bơ kêu. Rồi im lặng."),
    ("Cô Hằng bánh mì", "Lại Hội Thòng Lọng đấy cháu ạ. Chúng gom chó cả khu phố cổ, nhốt lồng rải rác rồi đêm mới chở đi."),
    ("Cô Hằng bánh mì", "Báo công an thì phải có bằng chứng. Còn muốn cứu con chó thì phải nhanh, trước khi trời sáng."),
    (HERO, "Bơ ơi, chờ tao. Tao đến đây."),
];

pub struct Chapter {
    pub title: &'static str,
    pub place: &'static str,
    pub brief: &'static str,
    pub tip: &'static str,
    pub after: &'static [Line],
}

pub const CHAPTERS: &[Chapter] = &[
    Chapter {
        title: "Đêm mất Bơ",
        place: "Phố Hàng Mã",
        brief: "Bọn trộm để một lồng chó trong ngõ quanh phố Hàng Mã. Cứu chú chó trong đó và thoát ra ở Ô Quan Chưởng.",
        tip: "Đẩy nhẹ cần điều khiển (hoặc giữ Shift) để đi lén: chậm nhưng không gây tiếng động. Vùng hình quạt là tầm nhìn của bọn trộm.",
        after: &[
            ("", "Không phải Bơ. Nhưng chú chó nhỏ run rẩy dụi đầu vào tay Minh."),
            (HERO, "Không sao rồi. Mày về phòng trọ với tao trước đã."),
            ("Cô Hằng bánh mì", "Cháu giỏi đấy. Cô nghe nói tối mai bọn chúng họp nhau ở phố Tạ Hiện. Đứa nào cũng giữ một cuốn sổ ghi chỗ nhốt chó."),
        ],
    },
    Chapter {
        title: "Tin đồn phố Tạ Hiện",
        place: "Phố Tạ Hiện",
        brief: "Lấy cuốn sổ ghi chép của bọn trộm ở quanh phố Tạ Hiện và cứu một chú chó.",
        tip: "Ném dép tổ ong để làm choáng, hoặc ném ra xa để dụ bọn trộm đến chỗ có tiếng động. Lẫn vào đám đông Tạ Hiện thì khó bị nhìn thấy.",
        after: &[
            ("", "Cuốn sổ nhàu nát, chữ xấu như gà bới. Nhưng có một dòng gạch chân: \"Đồng Xuân, 2 lồng, chở sáng thứ bảy\"."),
            (HERO, "Hai lồng một lúc... Một dây xích thì không dắt nổi hai con."),
            ("Anh Tú đồng nát", "Thiếu đồ thì qua chỗ anh. Dây xích đôi, giày, áo khoác, có hết, miễn là có tiền."),
        ],
    },
    Chapter {
        title: "Chợ Đồng Xuân",
        place: "Chợ Đồng Xuân",
        brief: "Hai lồng chó quanh chợ Đồng Xuân. Cứu cả hai trong một chuyến (cần Dây xích đôi).",
        tip: "Xe máy không nhường ai cả, kể cả bọn trộm. Dụ chúng ra đường lớn đúng lúc là một cách hay.",
        after: &[
            ("", "Một con chó vẫn đeo vòng cổ có số điện thoại. Chủ nó khóc khi nghe Minh gọi."),
            ("Cô Hằng bánh mì", "Thằng cầm đầu khu này là Tư Lồng. Nó hay lảng vảng ở phố đường tàu. Chìa khóa kho của cả hội nằm trong túi nó."),
        ],
    },
    Chapter {
        title: "Phố đường tàu",
        place: "Phố đường tàu",
        brief: "Hạ gục và trói Tư Lồng để lấy chìa khóa kho. Cứu thêm hai chú chó nếu kịp.",
        tip: "Tư Lồng gồng người trước khi lao tới: tránh sang ngang. Lao vào tường là hắn tự choáng. Nghe chuông là tàu sắp qua, tránh xa đường ray.",
        after: &[
            ("Tư Lồng", "Mày... mày có biết đại ca Bảy là ai không?"),
            (HERO, "Không. Nhưng tao biết chó của tao đang ở đâu rồi."),
            ("", "Chìa khóa có khắc một chữ: KHO. Kho bãi dưới chân cầu Long Biên."),
        ],
    },
    Chapter {
        title: "Cầu Long Biên",
        place: "Kho bãi Long Biên",
        brief: "Cứu Bơ trong kho bãi dưới chân cầu và thoát qua cầu Long Biên. Bảy Thòng Lọng đang canh.",
        tip: "Cả hội đã báo động. Đi lén, dùng hết số dép, và đừng đánh nhau lâu: mục tiêu là Bơ.",
        after: &[
            ("", "Bơ nhận ra Minh từ xa, vẫy đuôi đến mức cả người lắc theo."),
            (HERO, "Về nhà thôi, Bơ."),
            ("", "Sáng hôm sau, công an phường đến kho bãi với cuốn sổ của Hội Thòng Lọng. Mười hai chú chó về với chủ."),
            ("Cô Hằng bánh mì", "Từ giờ phòng trọ của cháu là trạm cứu hộ cún của cả phố đấy nhé. Đêm nào cũng có việc đâu."),
        ],
    },
];

pub const FREE_RAID_TITLE: &str = "Đi tuần phố";
pub const FREE_RAID_BRIEF: &str = "Đi tuần quanh Hoàn Kiếm: cứu được bao nhiêu chó thì cứu, nhặt đồ bán lấy tiền nâng cấp. Càng lên cấp, bọn trộm càng đông.";

/// The difficulty curve: one row per story chapter.
#[derive(Clone, Copy, Debug)]
pub struct Difficulty {
    pub thieves: u32,
    pub vision: f32,
    pub fov_deg: f32,
    pub alert_rate: f32,
    pub time_s: f32,
    pub reinforcements: bool,
    pub train_every_s: f32,
}

pub const CURVE: &[Difficulty] = &[
    Difficulty { thieves: 2, vision: 5.0, fov_deg: 70.0, alert_rate: 0.9, time_s: 420.0, reinforcements: false, train_every_s: 90.0 },
    Difficulty { thieves: 4, vision: 6.0, fov_deg: 75.0, alert_rate: 1.0, time_s: 390.0, reinforcements: true, train_every_s: 80.0 },
    Difficulty { thieves: 5, vision: 6.5, fov_deg: 80.0, alert_rate: 1.1, time_s: 390.0, reinforcements: true, train_every_s: 70.0 },
    Difficulty { thieves: 6, vision: 7.0, fov_deg: 85.0, alert_rate: 1.2, time_s: 360.0, reinforcements: true, train_every_s: 35.0 },
    Difficulty { thieves: 7, vision: 7.5, fov_deg: 90.0, alert_rate: 1.3, time_s: 360.0, reinforcements: true, train_every_s: 60.0 },
];

/// Free patrols get harder with each level.
pub fn free_difficulty(level: u32) -> Difficulty {
    let n = level.min(12) as f32;
    Difficulty {
        thieves: (3 + level).min(10),
        vision: (6.0 + 0.25 * n).min(8.5),
        fov_deg: 80.0,
        alert_rate: (1.0 + 0.05 * n).min(1.5),
        time_s: 360.0,
        reinforcements: true,
        train_every_s: 60.0,
    }
}

/// Exits open in each chapter (free patrols open two at random).
pub fn chapter_exits(ch: usize) -> &'static [ExitId] {
    match ch {
        0 => &[ExitId::QuanChuong],
        1 => &[ExitId::QuanChuong, ExitId::Ga],
        2 => &[ExitId::QuanChuong, ExitId::LongBien],
        3 => &[ExitId::Ga, ExitId::NhaHat],
        _ => &[ExitId::LongBien],
    }
}

pub const REWARD_CHAPTER: &[u32] = &[50, 60, 80, 100, 200];
pub const REWARD_PER_DOG: u32 = 10;

pub struct Breed {
    pub name: &'static str,
    pub coat: u32,
    pub ear: u32,
}

pub const BREEDS: &[Breed] = &[
    Breed { name: "Chó cỏ", coat: 0xd9a45b, ear: 0xa8763a },
    Breed { name: "Phú Quốc", coat: 0x6b4a32, ear: 0x3d2a1c },
    Breed { name: "Bắc Hà", coat: 0xe8dcc8, ear: 0xb8a58a },
    Breed { name: "H'Mông cộc đuôi", coat: 0x2f2a27, ear: 0x1a1715 },
    Breed { name: "Poodle", coat: 0xf2e6d8, ear: 0xd8c4ae },
    Breed { name: "Corgi", coat: 0xe39a4f, ear: 0xffffff },
    Breed { name: "Shiba", coat: 0xd98c3f, ear: 0xf4e1c6 },
    Breed { name: "Chó Lài", coat: 0xbfbfbf, ear: 0x5c5c5c },
];

pub const DOG_NAMES: &[&str] =
    &["Mực", "Vàng", "Lu", "Ki", "Bông", "Đốm", "Mít", "Na", "Tôm", "Xoài", "Cốm", "Sữa", "Bin", "Mochi", "Gạo", "Khoai", "Ớt", "Tiêu", "Đậu", "Muối"];

/// The breed Bơ is (chó cỏ) and his place in the dog book.
pub const PUPPY_BREED: usize = 0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpgradeKind {
    Shoes,
    Bag,
    Jacket,
    Leash,
    Pouch,
}

pub struct Upgrade {
    pub kind: UpgradeKind,
    pub name: &'static str,
    pub what: &'static str,
    /// Price of each level, in thousand đồng.
    pub prices: &'static [u32],
}

pub const UPGRADES: &[Upgrade] = &[
    Upgrade { kind: UpgradeKind::Leash, name: "Dây xích", what: "Dắt thêm một chú chó mỗi chuyến", prices: &[60, 150] },
    Upgrade { kind: UpgradeKind::Shoes, name: "Giày thể thao", what: "Chạy nhanh hơn 6% mỗi cấp", prices: &[40, 90, 160] },
    Upgrade { kind: UpgradeKind::Jacket, name: "Áo khoác dày", what: "Thêm 30 máu mỗi cấp", prices: &[50, 120] },
    Upgrade { kind: UpgradeKind::Bag, name: "Balo to", what: "Thêm 3 ô trong balo mỗi cấp", prices: &[40, 100] },
    Upgrade { kind: UpgradeKind::Pouch, name: "Túi dép", what: "Mang thêm 3 chiếc dép mỗi cấp", prices: &[30, 70] },
];

pub const SHOPKEEPER: &str = "Anh Tú đồng nát";

/// What a crate can hold, with its weight in the draw.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LootKind {
    Cash(u32),
    Scrap(u32),
    Slippers(u32),
    Bread,
}

pub const LOOT_TABLE: &[(LootKind, u32)] = &[
    (LootKind::Cash(5), 18),
    (LootKind::Cash(10), 12),
    (LootKind::Cash(20), 5),
    (LootKind::Scrap(8), 16),
    (LootKind::Scrap(15), 8),
    (LootKind::Slippers(2), 18),
    (LootKind::Bread, 14),
];

pub fn loot_name(k: LootKind) -> String {
    match k {
        LootKind::Cash(v) => format!("+{v}k tiền lẻ"),
        LootKind::Scrap(v) => format!("Đồng nát ({v}k)"),
        LootKind::Slippers(n) => format!("+{n} dép tổ ong"),
        LootKind::Bread => "Bánh mì: +30 máu".to_string(),
    }
}

pub mod ui {
    pub const PLAY: &str = "Chơi";
    pub const CONTINUE: &str = "Chơi tiếp";
    pub const NEW_GAME: &str = "Chơi lại từ đầu";
    pub const CONFIRM_NEW: &str = "Xóa tiến trình và chơi lại?";
    pub const YES: &str = "Xóa và chơi lại";
    pub const NO: &str = "Thôi";
    pub const SKIP: &str = "Bỏ qua";
    pub const TAP: &str = "Chạm để tiếp";
    pub const HIDEOUT: &str = "Phòng trọ";
    pub const RESCUE_STATION: &str = "Trạm cứu hộ cún";
    pub const SHOP: &str = "Cửa hàng";
    pub const DOGS: &str = "Sổ cún";
    pub const BACK: &str = "Quay lại";
    pub const GO: &str = "Xuất phát";
    pub const BUY: &str = "Mua";
    pub const MAXED: &str = "Tối đa";
    pub const PAUSED: &str = "Tạm dừng";
    pub const RESUME: &str = "Chơi tiếp";
    pub const ABANDON: &str = "Bỏ chuyến (mất balo)";
    pub const EXITS: &str = "Điểm thoát";
    pub const ATTACK: &str = "Đánh";
    pub const THROW: &str = "Dép";
    pub const ROLL: &str = "Lăn";
    pub const OPEN_CAGE: &str = "Mở lồng";
    pub const TIE: &str = "Trói";
    pub const EXTRACTING: &str = "Đang thoát…";
    pub const SUCCESS: &str = "THOÁT AN TOÀN";
    pub const CAUGHT: &str = "BỊ TÓM";
    pub const TIMEOUT: &str = "TRỜI SÁNG MẤT RỒI";
    pub const RETREAT: &str = "Rút lui an toàn (chưa xong nhiệm vụ)";
    pub const TO_HIDEOUT: &str = "Về phòng trọ";
    pub const BAG_FULL: &str = "Balo đầy";
    pub const LEASH_FULL: &str = "Hết dây xích";
    pub const REINFORCE: &str = "Bọn trộm gọi thêm đồng bọn!";
    pub const TRAIN: &str = "Tàu sắp qua! Tránh xa đường ray";
    pub const SPOTTED: &str = "!";
    pub const KEYS: &str = "WASD đi · Shift lén · J đánh · K ném dép · Space lăn · giữ E mở/trói · Esc dừng";
    pub const NO_DOGS_YET: &str = "Chưa cứu được chú chó nào. Đi thôi!";
}
