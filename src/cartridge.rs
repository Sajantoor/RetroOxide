use std::collections::HashMap;
use std::fs;
use std::io;

use num_enum;

#[derive(Debug)]
pub struct Cartridge {
    file_name: String,
    rom_size: u32,
    rom_data: Vec<u8>,
    pub rom_header: RomHeader,
}

// From 0x0100 - 0x014F
#[derive(Debug)]
pub struct RomHeader {
    logo: [u8; 48],                    // 0x0104 - 0x0133
    title: String,                     // 0x0134 - 0x0143
    cgb_flag: bool,                    // 0x0143 - ignored by us
    manufacturer_code: Option<u64>, // 0x013F - 0x0143  was part of the title, in new Cartridges contain a 4 character code in ascii
    new_licensee_code: Option<String>, // 0x0144 - 0x0145 2 character ASCII
    sgb_flag: bool,                 // 0x0146
    cartridge_type: u8,             // 0x0147
    rom_size: u8,                   // 0x0148 32 KiB × (1 << <value>)
    ram_size: u8,                   // 0x0149
    destination_code: DestinationCode, //0x014A
    old_license_code: OldLicenseCode, // 0x14B
    mask_rom_version_number: u8,    // 0x014C
    header_checksum: u8,            // 0x014D
    global_checksum: u16,           // 0x014E - 0x014F
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, num_enum::TryFromPrimitive)]
#[repr(u8)]
enum DestinationCode {
    Japan = 0x00,
    Overseas = 0x01,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, num_enum::TryFromPrimitive)]
#[repr(u8)]
enum OldLicenseCode {
    None = 0x00,
    Nintendo = 0x01,
    Capcom = 0x08,
    HotB = 0x09,
    Jaleco = 0x0A,
    CoconutsJapan = 0x0B,
    EliteSystems = 0x0C,
    EaElectronicArts = 0x13,
    HudsonSoft = 0x18,
    ItcEntertainment = 0x19,
    Yanoman = 0x1A,
    JapanClary = 0x1D,
    VirginGamesLtd3 = 0x1F,
    PcmComplete = 0x24,
    SanX = 0x25,
    Kemco = 0x28,
    SetaCorporation = 0x29,
    Infogrames5 = 0x30,
    Nintendo2 = 0x31,
    Bandai = 0x32,
    NewLicenseCode = 0x33,
    Konami = 0x34,
    HectorSoft = 0x35,
    Capcom2 = 0x38,
    Banpresto = 0x39,
    EntertainmentInteractiveStub = 0x3C,
    Gremlin = 0x3E,
    UbiSoft1 = 0x41,
    Atlus = 0x42,
    MalibuInteractive = 0x44,
    Angel = 0x46,
    SpectrumHoloByte = 0x47,
    Irem = 0x49,
    VirginGamesLtd3_2 = 0x4A,
    MalibuInteractive2 = 0x4D,
    UsGold = 0x4F,
    Absolute = 0x50,
    AcclaimEntertainment = 0x51,
    Activision = 0x52,
    SammyUsaCorporation = 0x53,
    GameTek = 0x54,
    ParkPlace13 = 0x55,
    Ljn = 0x56,
    Matchbox = 0x57,
    MiltonBradleyCompany = 0x59,
    Mindscape = 0x5A,
    Romstar = 0x5B,
    NaxatSoft14 = 0x5C,
    Tradewest = 0x5D,
    TitusInteractive = 0x60,
    VirginGamesLtd3_3 = 0x61,
    OceanSoftware = 0x67,
    EaElectronicArts2 = 0x69,
    EliteSystems2 = 0x6E,
    ElectroBrain = 0x6F,
    Infogrames5_2 = 0x70,
    InterplayEntertainment = 0x71,
    Broderbund = 0x72,
    SculpturedSoftware6 = 0x73,
    TheSalesCurveLimited7 = 0x75,
    Thq = 0x78,
    Accolade15 = 0x79,
    TriffixEntertainment = 0x7A,
    MicroProse = 0x7C,
    Kemco2 = 0x7F,
    MisawaEntertainment = 0x80,
    LozcG = 0x83,
    TokumaShoten = 0x86,
    BulletProofSoftware2 = 0x8B,
    VicTokaiCorp16 = 0x8C,
    ApeInc17 = 0x8E,
    IMax18 = 0x8F,
    ChunsoftCo8 = 0x91,
    VideoSystem = 0x92,
    TsubarayaProductions = 0x93,
    Varie = 0x95,
    Yonezawa19SPal = 0x96,
    Kemco3 = 0x97,
    Arc = 0x99,
    NihonBussan = 0x9A,
    Tecmo = 0x9B,
    Imagineer = 0x9C,
    Banpresto2 = 0x9D,
    Nova = 0x9F,
    HoriElectric = 0xA1,
    Bandai2 = 0xA2,
    Konami2 = 0xA4,
    Kawada = 0xA6,
    Takara = 0xA7,
    TechnosJapan = 0xA9,
    Broderbund2 = 0xAA,
    ToeiAnimation = 0xAC,
    Toho = 0xAD,
    Namco = 0xAF,
    AcclaimEntertainment2 = 0xB0,
    AsciiCorporationOrNexsoft = 0xB1,
    Bandai3 = 0xB2,
    SquareEnix = 0xB4,
    HalLaboratory = 0xB6,
    Snk = 0xB7,
    PonyCanyon = 0xB9,
    CultureBrain = 0xBA,
    Sunsoft = 0xBB,
    SonyImagesoft = 0xBD,
    SammyCorporation = 0xBF,
    Taito = 0xC0,
    Kemco4 = 0xC2,
    Square = 0xC3,
    TokumaShoten2 = 0xC4,
    DataEast = 0xC5,
    TonkinHouse = 0xC6,
    Koei = 0xC8,
    Ufl = 0xC9,
    UltraGames = 0xCA,
    VapInc = 0xCB,
    UseCorporation = 0xCC,
    Meldac = 0xCD,
    PonyCanyon2 = 0xCE,
    Angel2 = 0xCF,
    Taito2 = 0xD0,
    SofelSoftwareEngineeringLab = 0xD1,
    Quest = 0xD2,
    SigmaEnterprises = 0xD3,
    AskKodanshaCo = 0xD4,
    NaxatSoft14_2 = 0xD6,
    CopyaSystem = 0xD7,
    Banpresto3 = 0xD9,
    Tomy = 0xDA,
    Ljn2 = 0xDB,
    NipponComputerSystems = 0xDD,
    HumanEnt = 0xDE,
    Altron = 0xDF,
    Jaleco2 = 0xE0,
    TowaChiki = 0xE1,
    YutakaNeedsMoreInfo = 0xE2,
    Varie2 = 0xE3,
    Epoch = 0xE5,
    Athena = 0xE7,
    AsmikAceEntertainment = 0xE8,
    Natsume = 0xE9,
    KingRecords = 0xEA,
    Atlus2 = 0xEB,
    EpicSonyRecords = 0xEC,
    Igs = 0xEE,
    AWave = 0xF0,
    ExtremeEntertainment = 0xF3,
    Ljn3 = 0xFF,
}

use std::sync::LazyLock;
// TODO: There might be a better way to do this, this works for now but lifetimes are kind of yucky and I don't want to deal with them here...
static NEW_LICENSE_CODES: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("00", "None"),
        ("01", "NintendoResearchAndDevelopment1"),
        ("08", "Capcom"),
        ("13", "EaElectronicArts"),
        ("18", "HudsonSoft"),
        ("19", "Bai"),
        ("20", "Kss"),
        ("22", "PlanningOfficeWada"),
        ("24", "PcmComplete"),
        ("25", "SanX"),
        ("28", "Kemco"),
        ("29", "SetaCorporation"),
        ("30", "Viacom"),
        ("31", "Nintendo"),
        ("32", "Bandai"),
        ("33", "OceanSoftwareAcclaimEntertainment"),
        ("34", "Konami"),
        ("35", "HectorSoft"),
        ("37", "Taito"),
        ("38", "HudsonSoft2"),
        ("39", "Banpresto"),
        ("41", "UbiSoft1"),
        ("42", "Atlus"),
        ("44", "MalibuInteractive"),
        ("46", "Angel"),
        ("47", "BulletProofSoftware2"),
        ("49", "Irem"),
        ("50", "Absolute"),
        ("51", "AcclaimEntertainment"),
        ("52", "Activision"),
        ("53", "SammyUsaCorporation"),
        ("54", "Konami2"),
        ("55", "HiTechExpressions"),
        ("56", "Ljn"),
        ("57", "Matchbox"),
        ("58", "Mattel"),
        ("59", "MiltonBradleyCompany"),
        ("60", "TitusInteractive"),
        ("61", "VirginGamesLtd3"),
        ("64", "LucasfilmGames4"),
        ("67", "OceanSoftware"),
        ("69", "EaElectronicArts2"),
        ("70", "Infogrames5"),
        ("71", "InterplayEntertainment"),
        ("72", "Broderbund"),
        ("73", "SculpturedSoftware6"),
        ("75", "TheSalesCurveLimited7"),
        ("78", "Thq"),
        ("79", "Accolade8"),
        ("80", "MisawaEntertainment"),
        ("83", "LozcG"),
        ("86", "TokumaShoten"),
        ("87", "TsukudaOriginal"),
        ("91", "ChunsoftCo9"),
        ("92", "VideoSystem"),
        ("93", "OceanSoftwareAcclaimEntertainment2"),
        ("95", "Varie"),
        ("96", "Yonezawa10SPal"),
        ("97", "Kaneko"),
        ("99", "PackInVideo"),
        ("9H", "BottomUp"),
        ("A4", "KonamiYuGiOh"),
        ("BL", "Mto"),
        ("DK", "Kodansha"),
    ])
});

enum NewLicenseCode {}

impl Cartridge {
    pub fn from_file(path: &str) -> io::Result<Self> {
        let rom_data = fs::read(path)?;
        let rom_size = rom_data.len() as u32;

        let rom_header = RomHeader::parse(&rom_data)?;

        Ok(Self {
            file_name: path.to_string(),
            rom_size,
            rom_data,
            rom_header,
        })
    }

    pub fn validate_checksum() -> bool {
        panic!("Not implemented yet");
    }
}

impl RomHeader {
    pub fn parse(rom: &Vec<u8>) -> io::Result<Self> {
        let logo = rom[0x0104..(0x0133 + 1)].try_into().unwrap();

        let title_bytes = &rom[0x0134..(0x0143 + 1)];
        let title = String::from_utf8_lossy(title_bytes)
            .trim_matches('\0')
            .to_string();

        let cgb_flag = rom[0x0143] == 0x80 || rom[0x0143] == 0xC0;
        let manufacturer_code = None;

        let binding = String::from_utf8((&rom[0x0144..(0x0145 + 1)]).to_vec())
            .expect("Invalid new license code");

        let new_licensee_code_raw_ascii = binding.as_str();

        let new_licensee_code = NEW_LICENSE_CODES
            .get(new_licensee_code_raw_ascii)
            .map(|&code| code.to_string());

        let sgb_flag = rom[0x0146] == 0x03;
        let cartridge_type = rom[0x0147];
        let rom_size = rom[0x0148];
        let ram_size = rom[0x0149];

        let destination_code =
            DestinationCode::try_from(rom[0x014A]).unwrap_or(DestinationCode::Overseas);

        let old_license_code =
            OldLicenseCode::try_from(rom[0x014B]).unwrap_or(OldLicenseCode::None);

        let mask_rom_version_number = rom[0x014C];
        let header_checksum = rom[0x014D];
        let global_checksum = u16::from_be_bytes(rom[0x014E..(0x014F + 1)].try_into().unwrap());

        return Ok(Self {
            logo,
            title,
            cgb_flag,
            manufacturer_code,
            sgb_flag,
            new_licensee_code,
            cartridge_type,
            rom_size,
            ram_size,
            destination_code,
            old_license_code,
            mask_rom_version_number,
            header_checksum,
            global_checksum,
        });
    }
}
