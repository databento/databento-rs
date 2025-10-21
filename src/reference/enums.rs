use std::str::FromStr;

use serde::{de, Deserialize, Deserializer, Serialize};

use crate::{Error, Result};

/// A corporate actions action.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Action {
    /// Cancelled.
    Cancelled = b'C',
    /// Deleted.
    Deleted = b'D',
    /// Inserted.
    Inserted = b'I',
    /// Payment details cancelled by issuer.
    PaymentDetailsCancelledByIssuer = b'P',
    /// Payment details deleted by supplier.
    PaymentDetailsDeletedBySupplier = b'Q',
    /// Updated.
    Updated = b'U',
}
impl From<Action> for u8 {
    fn from(value: Action) -> u8 {
        value as u8
    }
}

impl From<Action> for char {
    fn from(value: Action) -> char {
        u8::from(value) as char
    }
}

impl TryFrom<u8> for Action {
    type Error = Error;

    fn try_from(value: u8) -> std::result::Result<Self, Error> {
        match value {
            b'C' => Ok(Self::Cancelled),
            b'D' => Ok(Self::Deleted),
            b'I' => Ok(Self::Inserted),
            b'P' => Ok(Self::PaymentDetailsCancelledByIssuer),
            b'Q' => Ok(Self::PaymentDetailsDeletedBySupplier),
            b'U' => Ok(Self::Updated),
            _ => Err(Error::bad_arg(
                "value",
                format!("no Action variant associated with {value:?}"),
            )),
        }
    }
}

impl<'de> Deserialize<'de> for Action {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let c = char::deserialize(deserializer)?;
        let u = u8::try_from(c).map_err(de::Error::custom)?;
        Action::try_from(u).map_err(de::Error::custom)
    }
}

impl Serialize for Action {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        (*self as u8 as char).serialize(serializer)
    }
}

/// The adjustment status.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum AdjustmentStatus {
    /// Apply.
    Apply = b'A',
    /// Rescind.
    Rescind = b'R',
    /// Pending.
    Pending = b'P',
}
impl From<AdjustmentStatus> for u8 {
    fn from(value: AdjustmentStatus) -> u8 {
        value as u8
    }
}

impl From<AdjustmentStatus> for char {
    fn from(value: AdjustmentStatus) -> char {
        u8::from(value) as char
    }
}

impl TryFrom<u8> for AdjustmentStatus {
    type Error = Error;

    fn try_from(value: u8) -> std::result::Result<Self, Error> {
        match value {
            b'A' => Ok(Self::Apply),
            b'R' => Ok(Self::Rescind),
            b'P' => Ok(Self::Pending),
            _ => Err(Error::bad_arg(
                "value",
                format!("no AdjustmentStatus variant associated with {value:?}"),
            )),
        }
    }
}

impl<'de> Deserialize<'de> for AdjustmentStatus {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let c = char::deserialize(deserializer)?;
        let u = u8::try_from(c).map_err(de::Error::custom)?;
        AdjustmentStatus::try_from(u).map_err(de::Error::custom)
    }
}

impl Serialize for AdjustmentStatus {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        (*self as u8 as char).serialize(serializer)
    }
}

/// A country code.
///
/// Based ISO 3166-1 alpha-2 country codes with some unofficial extensions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Country {
    /// Supranational
    Aa,
    /// Andorra
    Ad,
    /// United Arab Emirates
    Ae,
    /// Afghanistan
    Af,
    /// Antigua and Barbuda
    Ag,
    /// Anguilla
    Ai,
    /// Albania
    Al,
    /// Armenia
    Am,
    /// Netherlands Antilles
    An,
    /// Angola
    Ao,
    /// Argentina
    Ar,
    /// American Samoa
    As,
    /// Austria
    At,
    /// Australia
    Au,
    /// Aruba
    Aw,
    /// Azerbaijan
    Az,
    /// Bosnia and Herzegovina
    Ba,
    /// Barbados
    Bb,
    /// Bangladesh
    Bd,
    /// Belgium
    Be,
    /// Burkina Faso
    Bf,
    /// Bulgaria
    Bg,
    /// Bahrain
    Bh,
    /// Burundi
    Bi,
    /// Benin
    Bj,
    /// Bermuda
    Bm,
    /// Brunei Darussalam
    Bn,
    /// Bolivia
    Bo,
    /// Brazil
    Br,
    /// Bahamas
    Bs,
    /// Bhutan
    Bt,
    /// Bouvet Island
    Bv,
    /// Botswana
    Bw,
    /// Belarus
    By,
    /// Belize
    Bz,
    /// Canada
    Ca,
    /// Cocos Islands
    Cc,
    /// Congo Democratic Republic
    Cd,
    /// Central African Republic
    Cf,
    /// Congo
    Cg,
    /// Switzerland
    Ch,
    /// Ivory Coast
    Ci,
    /// Cook Islands
    Ck,
    /// Chile
    Cl,
    /// Cameroon
    Cm,
    /// China
    Cn,
    /// Colombia
    Co,
    /// Costa Rica
    Cr,
    /// Czechoslovakia
    Cs,
    /// Cuba
    Cu,
    /// Cape Verde
    Cv,
    /// Curacao
    Cw,
    /// Christmas Islands
    Cx,
    /// Cyprus
    Cy,
    /// Czech Republic
    Cz,
    /// Germany
    De,
    /// Djibouti
    Dj,
    /// Denmark
    Dk,
    /// Dominica
    Dm,
    /// Dominican Republic
    Do,
    /// Algeria
    Dz,
    /// Ecuador
    Ec,
    /// Estonia
    Ee,
    /// Egypt
    Eg,
    /// Western Sahara
    Eh,
    /// Spain
    Es,
    /// Ethiopia
    Et,
    /// Europe
    Eu,
    /// Finland
    Fi,
    /// Fiji
    Fj,
    /// Falkland Islands
    Fk,
    /// Micronesia
    Fm,
    /// Faroe Islands
    Fo,
    /// France
    Fr,
    /// French Metropolitan
    Fx,
    /// Gabon
    Ga,
    /// United Kingdom
    Gb,
    /// Grenada
    Gd,
    /// Georgia
    Ge,
    /// French Guyana
    Gf,
    /// Guernsey
    Gg,
    /// Ghana
    Gh,
    /// Gibraltar
    Gi,
    /// Greenland
    Gl,
    /// Gambia
    Gm,
    /// Guinea
    Gn,
    /// Guadeloupe
    Gp,
    /// Equatorial Guinea
    Gq,
    /// Greece
    Gr,
    /// Guatemala
    Gt,
    /// Guam
    Gu,
    /// Guinea-Bissau
    Gw,
    /// Guyana
    Gy,
    /// Hong Kong
    Hk,
    /// Heard Island and McDonald Island
    Hm,
    /// Honduras
    Hn,
    /// Croatia
    Hr,
    /// Haiti
    Ht,
    /// Hungary
    Hu,
    /// Indonesia
    Id,
    /// Ireland
    Ie,
    /// Israel
    Il,
    /// Isle of Man
    Im,
    /// India
    In,
    /// British Indian Ocean territory
    Io,
    /// Iraq
    Iq,
    /// Iran
    Ir,
    /// Iceland
    Is,
    /// Italy
    It,
    /// Jersey
    Je,
    /// Jamaica
    Jm,
    /// Jordan
    Jo,
    /// Japan
    Jp,
    /// Kenya
    Ke,
    /// Kyrgyz Republic
    Kg,
    /// Cambodia
    Kh,
    /// Kiribati
    Ki,
    /// Comoros
    Km,
    /// Saint Kitts and Nevis
    Kn,
    /// Korea (North)
    Kp,
    /// Korea (South)
    Kr,
    /// Kuwait
    Kw,
    /// Cayman Islands
    Ky,
    /// Kazakhstan
    Kz,
    /// Laos
    La,
    /// Lebanon
    Lb,
    /// Saint Lucia
    Lc,
    /// Liechtenstein
    Li,
    /// Sri Lanka
    Lk,
    /// Liberia
    Lr,
    /// Lesotho
    Ls,
    /// Lithuania
    Lt,
    /// Luxembourg
    Lu,
    /// Latvia
    Lv,
    /// Libya
    Ly,
    /// Morocco
    Ma,
    /// Monaco
    Mc,
    /// Moldova
    Md,
    /// Montenegro
    Me,
    /// Madagascar
    Mg,
    /// Marshall Islands
    Mh,
    /// North Macedonia
    Mk,
    /// Mali
    Ml,
    /// Myanmar
    Mm,
    /// Mongolia
    Mn,
    /// Macao
    Mo,
    /// Northern Mariana Islands
    Mp,
    /// Martinique
    Mq,
    /// Mauritania
    Mr,
    /// Montserrat
    Ms,
    /// Malta
    Mt,
    /// Mauritius
    Mu,
    /// Maldives
    Mv,
    /// Malawi
    Mw,
    /// Mexico
    Mx,
    /// Malaysia
    My,
    /// Mozambique
    Mz,
    /// Namibia
    Na,
    /// New Caledonia
    Nc,
    /// Niger
    Ne,
    /// Norfolk Island
    Nf,
    /// Nigeria
    Ng,
    /// Nicaragua
    Ni,
    /// Netherlands
    Nl,
    /// Norway
    No,
    /// Nepal
    Np,
    /// Nauru
    Nr,
    /// Niue
    Nu,
    /// New Zealand
    Nz,
    /// Oman
    Om,
    /// Panama
    Pa,
    /// Peru
    Pe,
    /// French Polynesia
    Pf,
    /// Papua New Guinea
    Pg,
    /// Philippines
    Ph,
    /// Pakistan
    Pk,
    /// Poland
    Pl,
    /// Saint Pierre and Miquelon
    Pm,
    /// Pitcairn
    Pn,
    /// Puerto Rico
    Pr,
    /// Palestine
    Ps,
    /// Portugal
    Pt,
    /// Palau
    Pw,
    /// Paraguay
    Py,
    /// Qatar
    Qa,
    /// Reunion
    Re,
    /// Romania
    Ro,
    /// Serbia
    Rs,
    /// Russia
    Ru,
    /// Rwanda
    Rw,
    /// Saudi Arabia
    Sa,
    /// Solomon Islands
    Sb,
    /// Seychelles
    Sc,
    /// Sudan
    Sd,
    /// Sweden
    Se,
    /// Singapore
    Sg,
    /// Saint Helena
    Sh,
    /// Slovenia
    Si,
    /// Svalbard and Jan Mayen
    Sj,
    /// Slovak Republic
    Sk,
    /// Sierra Leone
    Sl,
    /// San Marino
    Sm,
    /// Senegal
    Sn,
    /// Somalia
    So,
    /// Suriname
    Sr,
    /// Sao Tome and Principe
    St,
    /// USSR
    Su,
    /// El Salvador
    Sv,
    /// Syria
    Sy,
    /// Eswatini
    Sz,
    /// Turks and Caicos Islands
    Tc,
    /// Chad
    Td,
    /// French Southern Territories
    Tf,
    /// Togo
    Tg,
    /// Thailand
    Th,
    /// Tajikistan
    Tj,
    /// Tokelau
    Tk,
    /// Turkmenistan
    Tm,
    /// Tunisia
    Tn,
    /// Tonga
    To,
    /// East Timor
    Tp,
    /// Turkey
    Tr,
    /// Trinidad and Tobago
    Tt,
    /// Tuvalu
    Tv,
    /// Taiwan
    Tw,
    /// Tanzania
    Tz,
    /// Ukraine
    Ua,
    /// Uganda
    Ug,
    /// United States Minor Outlying
    Um,
    /// United States of America
    Us,
    /// Uruguay
    Uy,
    /// Uzbekistan
    Uz,
    /// Vatican City
    Va,
    /// Saint Vincent and the Grenadines
    Vc,
    /// Venezuela
    Ve,
    /// UK Virgin Islands
    Vg,
    /// US Virgin Islands
    Vi,
    /// Vietnam
    Vn,
    /// Vanuatu
    Vu,
    /// Wallis and Futuna Islands
    Wf,
    /// Samoa
    Ws,
    /// Shanghai SC
    Xg,
    /// Hong Kong SC
    Xh,
    /// Shenzhen SC
    Xz,
    /// Yemen
    Ye,
    /// Mayotte
    Yt,
    /// South Africa
    Za,
    /// Zambia
    Zm,
    /// Zimbabwe
    Zw,
    /// Unclassified
    Zz,
}
impl Country {
    /// Converts a Country to its `str` code.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Aa => "AA",
            Self::Ad => "AD",
            Self::Ae => "AE",
            Self::Af => "AF",
            Self::Ag => "AG",
            Self::Ai => "AI",
            Self::Al => "AL",
            Self::Am => "AM",
            Self::An => "AN",
            Self::Ao => "AO",
            Self::Ar => "AR",
            Self::As => "AS",
            Self::At => "AT",
            Self::Au => "AU",
            Self::Aw => "AW",
            Self::Az => "AZ",
            Self::Ba => "BA",
            Self::Bb => "BB",
            Self::Bd => "BD",
            Self::Be => "BE",
            Self::Bf => "BF",
            Self::Bg => "BG",
            Self::Bh => "BH",
            Self::Bi => "BI",
            Self::Bj => "BJ",
            Self::Bm => "BM",
            Self::Bn => "BN",
            Self::Bo => "BO",
            Self::Br => "BR",
            Self::Bs => "BS",
            Self::Bt => "BT",
            Self::Bv => "BV",
            Self::Bw => "BW",
            Self::By => "BY",
            Self::Bz => "BZ",
            Self::Ca => "CA",
            Self::Cc => "CC",
            Self::Cd => "CD",
            Self::Cf => "CF",
            Self::Cg => "CG",
            Self::Ch => "CH",
            Self::Ci => "CI",
            Self::Ck => "CK",
            Self::Cl => "CL",
            Self::Cm => "CM",
            Self::Cn => "CN",
            Self::Co => "CO",
            Self::Cr => "CR",
            Self::Cs => "CS",
            Self::Cu => "CU",
            Self::Cv => "CV",
            Self::Cw => "CW",
            Self::Cx => "CX",
            Self::Cy => "CY",
            Self::Cz => "CZ",
            Self::De => "DE",
            Self::Dj => "DJ",
            Self::Dk => "DK",
            Self::Dm => "DM",
            Self::Do => "DO",
            Self::Dz => "DZ",
            Self::Ec => "EC",
            Self::Ee => "EE",
            Self::Eg => "EG",
            Self::Eh => "EH",
            Self::Es => "ES",
            Self::Et => "ET",
            Self::Eu => "EU",
            Self::Fi => "FI",
            Self::Fj => "FJ",
            Self::Fk => "FK",
            Self::Fm => "FM",
            Self::Fo => "FO",
            Self::Fr => "FR",
            Self::Fx => "FX",
            Self::Ga => "GA",
            Self::Gb => "GB",
            Self::Gd => "GD",
            Self::Ge => "GE",
            Self::Gf => "GF",
            Self::Gg => "GG",
            Self::Gh => "GH",
            Self::Gi => "GI",
            Self::Gl => "GL",
            Self::Gm => "GM",
            Self::Gn => "GN",
            Self::Gp => "GP",
            Self::Gq => "GQ",
            Self::Gr => "GR",
            Self::Gt => "GT",
            Self::Gu => "GU",
            Self::Gw => "GW",
            Self::Gy => "GY",
            Self::Hk => "HK",
            Self::Hm => "HM",
            Self::Hn => "HN",
            Self::Hr => "HR",
            Self::Ht => "HT",
            Self::Hu => "HU",
            Self::Id => "ID",
            Self::Ie => "IE",
            Self::Il => "IL",
            Self::Im => "IM",
            Self::In => "IN",
            Self::Io => "IO",
            Self::Iq => "IQ",
            Self::Ir => "IR",
            Self::Is => "IS",
            Self::It => "IT",
            Self::Je => "JE",
            Self::Jm => "JM",
            Self::Jo => "JO",
            Self::Jp => "JP",
            Self::Ke => "KE",
            Self::Kg => "KG",
            Self::Kh => "KH",
            Self::Ki => "KI",
            Self::Km => "KM",
            Self::Kn => "KN",
            Self::Kp => "KP",
            Self::Kr => "KR",
            Self::Kw => "KW",
            Self::Ky => "KY",
            Self::Kz => "KZ",
            Self::La => "LA",
            Self::Lb => "LB",
            Self::Lc => "LC",
            Self::Li => "LI",
            Self::Lk => "LK",
            Self::Lr => "LR",
            Self::Ls => "LS",
            Self::Lt => "LT",
            Self::Lu => "LU",
            Self::Lv => "LV",
            Self::Ly => "LY",
            Self::Ma => "MA",
            Self::Mc => "MC",
            Self::Md => "MD",
            Self::Me => "ME",
            Self::Mg => "MG",
            Self::Mh => "MH",
            Self::Mk => "MK",
            Self::Ml => "ML",
            Self::Mm => "MM",
            Self::Mn => "MN",
            Self::Mo => "MO",
            Self::Mp => "MP",
            Self::Mq => "MQ",
            Self::Mr => "MR",
            Self::Ms => "MS",
            Self::Mt => "MT",
            Self::Mu => "MU",
            Self::Mv => "MV",
            Self::Mw => "MW",
            Self::Mx => "MX",
            Self::My => "MY",
            Self::Mz => "MZ",
            Self::Na => "NA",
            Self::Nc => "NC",
            Self::Ne => "NE",
            Self::Nf => "NF",
            Self::Ng => "NG",
            Self::Ni => "NI",
            Self::Nl => "NL",
            Self::No => "NO",
            Self::Np => "NP",
            Self::Nr => "NR",
            Self::Nu => "NU",
            Self::Nz => "NZ",
            Self::Om => "OM",
            Self::Pa => "PA",
            Self::Pe => "PE",
            Self::Pf => "PF",
            Self::Pg => "PG",
            Self::Ph => "PH",
            Self::Pk => "PK",
            Self::Pl => "PL",
            Self::Pm => "PM",
            Self::Pn => "PN",
            Self::Pr => "PR",
            Self::Ps => "PS",
            Self::Pt => "PT",
            Self::Pw => "PW",
            Self::Py => "PY",
            Self::Qa => "QA",
            Self::Re => "RE",
            Self::Ro => "RO",
            Self::Rs => "RS",
            Self::Ru => "RU",
            Self::Rw => "RW",
            Self::Sa => "SA",
            Self::Sb => "SB",
            Self::Sc => "SC",
            Self::Sd => "SD",
            Self::Se => "SE",
            Self::Sg => "SG",
            Self::Sh => "SH",
            Self::Si => "SI",
            Self::Sj => "SJ",
            Self::Sk => "SK",
            Self::Sl => "SL",
            Self::Sm => "SM",
            Self::Sn => "SN",
            Self::So => "SO",
            Self::Sr => "SR",
            Self::St => "ST",
            Self::Su => "SU",
            Self::Sv => "SV",
            Self::Sy => "SY",
            Self::Sz => "SZ",
            Self::Tc => "TC",
            Self::Td => "TD",
            Self::Tf => "TF",
            Self::Tg => "TG",
            Self::Th => "TH",
            Self::Tj => "TJ",
            Self::Tk => "TK",
            Self::Tm => "TM",
            Self::Tn => "TN",
            Self::To => "TO",
            Self::Tp => "TP",
            Self::Tr => "TR",
            Self::Tt => "TT",
            Self::Tv => "TV",
            Self::Tw => "TW",
            Self::Tz => "TZ",
            Self::Ua => "UA",
            Self::Ug => "UG",
            Self::Um => "UM",
            Self::Us => "US",
            Self::Uy => "UY",
            Self::Uz => "UZ",
            Self::Va => "VA",
            Self::Vc => "VC",
            Self::Ve => "VE",
            Self::Vg => "VG",
            Self::Vi => "VI",
            Self::Vn => "VN",
            Self::Vu => "VU",
            Self::Wf => "WF",
            Self::Ws => "WS",
            Self::Xg => "XG",
            Self::Xh => "XH",
            Self::Xz => "XZ",
            Self::Ye => "YE",
            Self::Yt => "YT",
            Self::Za => "ZA",
            Self::Zm => "ZM",
            Self::Zw => "ZW",
            Self::Zz => "ZZ",
        }
    }
}

impl AsRef<str> for Country {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::str::FromStr for Country {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "AA" => Ok(Self::Aa),
            "AD" => Ok(Self::Ad),
            "AE" => Ok(Self::Ae),
            "AF" => Ok(Self::Af),
            "AG" => Ok(Self::Ag),
            "AI" => Ok(Self::Ai),
            "AL" => Ok(Self::Al),
            "AM" => Ok(Self::Am),
            "AN" => Ok(Self::An),
            "AO" => Ok(Self::Ao),
            "AR" => Ok(Self::Ar),
            "AS" => Ok(Self::As),
            "AT" => Ok(Self::At),
            "AU" => Ok(Self::Au),
            "AW" => Ok(Self::Aw),
            "AZ" => Ok(Self::Az),
            "BA" => Ok(Self::Ba),
            "BB" => Ok(Self::Bb),
            "BD" => Ok(Self::Bd),
            "BE" => Ok(Self::Be),
            "BF" => Ok(Self::Bf),
            "BG" => Ok(Self::Bg),
            "BH" => Ok(Self::Bh),
            "BI" => Ok(Self::Bi),
            "BJ" => Ok(Self::Bj),
            "BM" => Ok(Self::Bm),
            "BN" => Ok(Self::Bn),
            "BO" => Ok(Self::Bo),
            "BR" => Ok(Self::Br),
            "BS" => Ok(Self::Bs),
            "BT" => Ok(Self::Bt),
            "BV" => Ok(Self::Bv),
            "BW" => Ok(Self::Bw),
            "BY" => Ok(Self::By),
            "BZ" => Ok(Self::Bz),
            "CA" => Ok(Self::Ca),
            "CC" => Ok(Self::Cc),
            "CD" => Ok(Self::Cd),
            "CF" => Ok(Self::Cf),
            "CG" => Ok(Self::Cg),
            "CH" => Ok(Self::Ch),
            "CI" => Ok(Self::Ci),
            "CK" => Ok(Self::Ck),
            "CL" => Ok(Self::Cl),
            "CM" => Ok(Self::Cm),
            "CN" => Ok(Self::Cn),
            "CO" => Ok(Self::Co),
            "CR" => Ok(Self::Cr),
            "CS" => Ok(Self::Cs),
            "CU" => Ok(Self::Cu),
            "CV" => Ok(Self::Cv),
            "CW" => Ok(Self::Cw),
            "CX" => Ok(Self::Cx),
            "CY" => Ok(Self::Cy),
            "CZ" => Ok(Self::Cz),
            "DE" => Ok(Self::De),
            "DJ" => Ok(Self::Dj),
            "DK" => Ok(Self::Dk),
            "DM" => Ok(Self::Dm),
            "DO" => Ok(Self::Do),
            "DZ" => Ok(Self::Dz),
            "EC" => Ok(Self::Ec),
            "EE" => Ok(Self::Ee),
            "EG" => Ok(Self::Eg),
            "EH" => Ok(Self::Eh),
            "ES" => Ok(Self::Es),
            "ET" => Ok(Self::Et),
            "EU" => Ok(Self::Eu),
            "FI" => Ok(Self::Fi),
            "FJ" => Ok(Self::Fj),
            "FK" => Ok(Self::Fk),
            "FM" => Ok(Self::Fm),
            "FO" => Ok(Self::Fo),
            "FR" => Ok(Self::Fr),
            "FX" => Ok(Self::Fx),
            "GA" => Ok(Self::Ga),
            "GB" => Ok(Self::Gb),
            "GD" => Ok(Self::Gd),
            "GE" => Ok(Self::Ge),
            "GF" => Ok(Self::Gf),
            "GG" => Ok(Self::Gg),
            "GH" => Ok(Self::Gh),
            "GI" => Ok(Self::Gi),
            "GL" => Ok(Self::Gl),
            "GM" => Ok(Self::Gm),
            "GN" => Ok(Self::Gn),
            "GP" => Ok(Self::Gp),
            "GQ" => Ok(Self::Gq),
            "GR" => Ok(Self::Gr),
            "GT" => Ok(Self::Gt),
            "GU" => Ok(Self::Gu),
            "GW" => Ok(Self::Gw),
            "GY" => Ok(Self::Gy),
            "HK" => Ok(Self::Hk),
            "HM" => Ok(Self::Hm),
            "HN" => Ok(Self::Hn),
            "HR" => Ok(Self::Hr),
            "HT" => Ok(Self::Ht),
            "HU" => Ok(Self::Hu),
            "ID" => Ok(Self::Id),
            "IE" => Ok(Self::Ie),
            "IL" => Ok(Self::Il),
            "IM" => Ok(Self::Im),
            "IN" => Ok(Self::In),
            "IO" => Ok(Self::Io),
            "IQ" => Ok(Self::Iq),
            "IR" => Ok(Self::Ir),
            "IS" => Ok(Self::Is),
            "IT" => Ok(Self::It),
            "JE" => Ok(Self::Je),
            "JM" => Ok(Self::Jm),
            "JO" => Ok(Self::Jo),
            "JP" => Ok(Self::Jp),
            "KE" => Ok(Self::Ke),
            "KG" => Ok(Self::Kg),
            "KH" => Ok(Self::Kh),
            "KI" => Ok(Self::Ki),
            "KM" => Ok(Self::Km),
            "KN" => Ok(Self::Kn),
            "KP" => Ok(Self::Kp),
            "KR" => Ok(Self::Kr),
            "KW" => Ok(Self::Kw),
            "KY" => Ok(Self::Ky),
            "KZ" => Ok(Self::Kz),
            "LA" => Ok(Self::La),
            "LB" => Ok(Self::Lb),
            "LC" => Ok(Self::Lc),
            "LI" => Ok(Self::Li),
            "LK" => Ok(Self::Lk),
            "LR" => Ok(Self::Lr),
            "LS" => Ok(Self::Ls),
            "LT" => Ok(Self::Lt),
            "LU" => Ok(Self::Lu),
            "LV" => Ok(Self::Lv),
            "LY" => Ok(Self::Ly),
            "MA" => Ok(Self::Ma),
            "MC" => Ok(Self::Mc),
            "MD" => Ok(Self::Md),
            "ME" => Ok(Self::Me),
            "MG" => Ok(Self::Mg),
            "MH" => Ok(Self::Mh),
            "MK" => Ok(Self::Mk),
            "ML" => Ok(Self::Ml),
            "MM" => Ok(Self::Mm),
            "MN" => Ok(Self::Mn),
            "MO" => Ok(Self::Mo),
            "MP" => Ok(Self::Mp),
            "MQ" => Ok(Self::Mq),
            "MR" => Ok(Self::Mr),
            "MS" => Ok(Self::Ms),
            "MT" => Ok(Self::Mt),
            "MU" => Ok(Self::Mu),
            "MV" => Ok(Self::Mv),
            "MW" => Ok(Self::Mw),
            "MX" => Ok(Self::Mx),
            "MY" => Ok(Self::My),
            "MZ" => Ok(Self::Mz),
            "NA" => Ok(Self::Na),
            "NC" => Ok(Self::Nc),
            "NE" => Ok(Self::Ne),
            "NF" => Ok(Self::Nf),
            "NG" => Ok(Self::Ng),
            "NI" => Ok(Self::Ni),
            "NL" => Ok(Self::Nl),
            "NO" => Ok(Self::No),
            "NP" => Ok(Self::Np),
            "NR" => Ok(Self::Nr),
            "NU" => Ok(Self::Nu),
            "NZ" => Ok(Self::Nz),
            "OM" => Ok(Self::Om),
            "PA" => Ok(Self::Pa),
            "PE" => Ok(Self::Pe),
            "PF" => Ok(Self::Pf),
            "PG" => Ok(Self::Pg),
            "PH" => Ok(Self::Ph),
            "PK" => Ok(Self::Pk),
            "PL" => Ok(Self::Pl),
            "PM" => Ok(Self::Pm),
            "PN" => Ok(Self::Pn),
            "PR" => Ok(Self::Pr),
            "PS" => Ok(Self::Ps),
            "PT" => Ok(Self::Pt),
            "PW" => Ok(Self::Pw),
            "PY" => Ok(Self::Py),
            "QA" => Ok(Self::Qa),
            "RE" => Ok(Self::Re),
            "RO" => Ok(Self::Ro),
            "RS" => Ok(Self::Rs),
            "RU" => Ok(Self::Ru),
            "RW" => Ok(Self::Rw),
            "SA" => Ok(Self::Sa),
            "SB" => Ok(Self::Sb),
            "SC" => Ok(Self::Sc),
            "SD" => Ok(Self::Sd),
            "SE" => Ok(Self::Se),
            "SG" => Ok(Self::Sg),
            "SH" => Ok(Self::Sh),
            "SI" => Ok(Self::Si),
            "SJ" => Ok(Self::Sj),
            "SK" => Ok(Self::Sk),
            "SL" => Ok(Self::Sl),
            "SM" => Ok(Self::Sm),
            "SN" => Ok(Self::Sn),
            "SO" => Ok(Self::So),
            "SR" => Ok(Self::Sr),
            "ST" => Ok(Self::St),
            "SU" => Ok(Self::Su),
            "SV" => Ok(Self::Sv),
            "SY" => Ok(Self::Sy),
            "SZ" => Ok(Self::Sz),
            "TC" => Ok(Self::Tc),
            "TD" => Ok(Self::Td),
            "TF" => Ok(Self::Tf),
            "TG" => Ok(Self::Tg),
            "TH" => Ok(Self::Th),
            "TJ" => Ok(Self::Tj),
            "TK" => Ok(Self::Tk),
            "TM" => Ok(Self::Tm),
            "TN" => Ok(Self::Tn),
            "TO" => Ok(Self::To),
            "TP" => Ok(Self::Tp),
            "TR" => Ok(Self::Tr),
            "TT" => Ok(Self::Tt),
            "TV" => Ok(Self::Tv),
            "TW" => Ok(Self::Tw),
            "TZ" => Ok(Self::Tz),
            "UA" => Ok(Self::Ua),
            "UG" => Ok(Self::Ug),
            "UM" => Ok(Self::Um),
            "US" => Ok(Self::Us),
            "UY" => Ok(Self::Uy),
            "UZ" => Ok(Self::Uz),
            "VA" => Ok(Self::Va),
            "VC" => Ok(Self::Vc),
            "VE" => Ok(Self::Ve),
            "VG" => Ok(Self::Vg),
            "VI" => Ok(Self::Vi),
            "VN" => Ok(Self::Vn),
            "VU" => Ok(Self::Vu),
            "WF" => Ok(Self::Wf),
            "WS" => Ok(Self::Ws),
            "XG" => Ok(Self::Xg),
            "XH" => Ok(Self::Xh),
            "XZ" => Ok(Self::Xz),
            "YE" => Ok(Self::Ye),
            "YT" => Ok(Self::Yt),
            "ZA" => Ok(Self::Za),
            "ZM" => Ok(Self::Zm),
            "ZW" => Ok(Self::Zw),
            "ZZ" => Ok(Self::Zz),
            _ => Err(Error::bad_arg(
                "s",
                format!("no Country variant associated with {s}"),
            )),
        }
    }
}

impl<'de> Deserialize<'de> for Country {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let str = String::deserialize(deserializer)?;
        FromStr::from_str(&str).map_err(de::Error::custom)
    }
}

impl Serialize for Country {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_str().serialize(serializer)
    }
}

/// A currency.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Currency {
    /// Andorra Peseta
    Adp,
    /// UAE Dirham
    Aed,
    /// Afghanis
    Afn,
    /// Albanian Lek
    All,
    /// Armenian Dram
    Amd,
    /// Netherlands Antilles Guilders
    Ang,
    /// Angola Kwanza
    Aoa,
    /// Argentine Peso
    Ars,
    /// Austrian Schilling
    Ats,
    /// Australian Dollar
    Aud,
    /// Aruban Guilder
    Awg,
    /// Azerbaijani Manat
    Azn,
    /// Convertible Marks
    Bam,
    /// Barbados Dollar
    Bbd,
    /// Bangladesh Taka
    Bdt,
    /// Belgian Franc
    Bef,
    /// Bulgarian Lev
    Bgn,
    /// Bahraini Dinar
    Bhd,
    /// Burundi Franc
    Bif,
    /// Bermuda Dollar
    Bmd,
    /// Brunei Dollar
    Bnd,
    /// Boliviano
    Bob,
    /// Mvdol
    Bov,
    /// Brazilian Real
    Brl,
    /// Bahamas Dollar
    Bsd,
    /// Bhutanese Ngultrum
    Btn,
    /// Botswana Pula
    Bwp,
    /// Belarusian Ruble (New)
    Byn,
    /// Belarussian Ruble
    Byr,
    /// Belize Dollar
    Bzd,
    /// Canadian Dollar
    Cad,
    /// Congolese Franc
    Cdf,
    /// Swiss Francs
    Chf,
    /// Chilean Unidad de Fomento
    Clf,
    /// Chilean Peso
    Clp,
    /// Chinese Yuan Renminbi
    Cny,
    /// Colombian Peso
    Cop,
    /// Colombian (Unidad de Valor Real)
    Cou,
    /// Costa Rican Colon
    Crc,
    /// Cuban Peso
    Cup,
    /// Cape Verde Escudo
    Cve,
    /// Cypriot Pound
    Cyp,
    /// Czech Koruna
    Czk,
    /// Deutschmark
    Dem,
    /// Djibouti Franc
    Djf,
    /// Danish Kroner
    Dkk,
    /// Dominican Peso
    Dop,
    /// Algerian Dinar
    Dzd,
    /// Ecuador Sucre
    Ecs,
    /// Estonian Kroon
    Eek,
    /// Egyptian Pound
    Egp,
    /// Eritrean Nakfa
    Ern,
    /// Spanish Pesetas
    Esp,
    /// Ethiopian Birr
    Etb,
    /// Euros
    Eur,
    /// Finnish Markka
    Fim,
    /// Fiji Dollar
    Fjd,
    /// Falklands Pounds
    Fkp,
    /// French Francs
    Frf,
    /// Pound Sterling
    Gbp,
    /// GB Pence
    Gbx,
    /// Georgian Lari
    Gel,
    /// Ghana Cedi
    Ghc,
    /// Ghanaian Cedi
    Ghs,
    /// Gibraltar Pounds
    Gip,
    /// Gambian Dalasi
    Gmd,
    /// Guinean Franc
    Gnf,
    /// Greek Drachma
    Grd,
    /// Guatemalan Quetzal
    Gtq,
    /// Guyana Dollar
    Gyd,
    /// Hong Kong Dollar
    Hkd,
    /// Honduras Lempira
    Hnl,
    /// Croatian Kuna
    Hrk,
    /// Haiti Gourde
    Htg,
    /// Hungarian Forint
    Huf,
    /// Indonesian Rupiah
    Idr,
    /// Irish Punt
    Iep,
    /// Israeli New Shekel
    Ils,
    /// Indian Rupees
    Inr,
    /// Iraqi Dinar
    Iqd,
    /// Iranian Rial
    Irr,
    /// Icelandic Krona
    Isk,
    /// Italian Lire
    Itl,
    /// Jersey Pounds
    Jep,
    /// Jamaican Dollar
    Jmd,
    /// Jordanian Dinar
    Jod,
    /// Japanese Yen
    Jpy,
    /// Kenyan Shilling
    Kes,
    /// Kyrgyzstan Som
    Kgs,
    /// Cambodian Riel
    Khr,
    /// Comoro Franc
    Kmf,
    /// North Korean Won
    Kpw,
    /// Korean Won
    Krw,
    /// Kuwaiti Dinar
    Kwd,
    /// Cayman Islands Dollar
    Kyd,
    /// Kazakhstan Tenge
    Kzt,
    /// Lao Liberation Kip
    Lak,
    /// Lebanese Pound
    Lbp,
    /// Sri Lankan Rupee
    Lkr,
    /// Liberian Dollar
    Lrd,
    /// Lesotho Loti
    Lsl,
    /// Lithuanian Litas
    Ltl,
    /// Luxembourg Francs
    Luf,
    /// Latvian Lats
    Lvl,
    /// Libyan Dinar
    Lyd,
    /// Moroccan Dirham
    Mad,
    /// Moldovan Leu
    Mdl,
    /// Malagasy Ariary
    Mga,
    /// Macedonian Denar
    Mkd,
    /// Myanmar Kyat
    Mmk,
    /// Mongolian Tugrik
    Mnt,
    /// Macau Pataca
    Mop,
    /// Mauritanian Ouguiya
    Mro,
    /// Maltese Lira
    Mtl,
    /// Mauritius Rupee
    Mur,
    /// Maldivian Rufiyaa
    Mvr,
    /// Malawi Kwacha
    Mwk,
    /// Mexican Nuevo Peso
    Mxn,
    /// Mexican Unidad de Inversion (UDI)
    Mxv,
    /// Malaysian Ringgit
    Myr,
    /// Mozambique Metical
    Mzn,
    /// Namibian Dollar
    Nad,
    /// Nigerian Naira
    Ngn,
    /// Nicaraguan Cordoba Oro
    Nio,
    /// Netherlands Guilder
    Nlg,
    /// Norwegian Krone
    Nok,
    /// Nepalese Rupee
    Npr,
    /// New Zealand Dollar
    Nzd,
    /// Omani Rial
    Omr,
    /// Panama Balboa
    Pab,
    /// Peruvian Nuevo Sol
    Pen,
    /// Papua New Guinea Kina
    Pgk,
    /// Philippines Peso
    Php,
    /// Pakistan Rupee
    Pkr,
    /// Polish Zloty (New)
    Pln,
    /// Portuguese Escudo
    Pte,
    /// Paraguay Guarani
    Pyg,
    /// Qatar Rial
    Qar,
    /// Romanian Leu
    Rol,
    /// Romanian Leu (New)
    Ron,
    /// Serbian Dinars
    Rsd,
    /// Russian Ruble (New)
    Rub,
    /// Russian Ruble
    Rur,
    /// Rwandan Franc
    Rwf,
    /// Saudi Arabian Riyal
    Sar,
    /// Solomon Islands Dollar
    Sbd,
    /// Seychelles Rupee
    Scr,
    /// Sudanese Dinar
    Sdd,
    /// Sudanese Pound
    Sdg,
    /// Swedish Kroner
    Sek,
    /// Singapore Dollar
    Sgd,
    /// St. Helena Pounds
    Shp,
    /// Slovenian Tolar
    Sit,
    /// Slovak Koruna
    Skk,
    /// Sierra Leone
    Sll,
    /// Somalia Shilling
    Sos,
    /// Surinam Dollar
    Srd,
    /// Sao Tome and Principe Dobra
    Std,
    /// El Salvador Colon
    Svc,
    /// Syrian Pound
    Syp,
    /// Swaziland Lilangeni
    Szl,
    /// Thai Baht
    Thb,
    /// Tajikistani Somoni
    Tjs,
    /// Turkmenistan Manat
    Tmm,
    /// Tunisian Dinar
    Tnd,
    /// Tonga Pa`anga
    Top,
    /// Turkish Lira
    Trl,
    /// Turkish Lira (New)
    Try,
    /// Trinidad and Tobago Dollar
    Ttd,
    /// Taiwan Dollar
    Twd,
    /// Tanzanian Shilling
    Tzs,
    /// Ukrainian Hryvnia
    Uah,
    /// Ugandan Shilling
    Ugx,
    /// US Dollar
    Usd,
    /// US Cents
    Usx,
    /// Uruguay Peso (Index Linked)
    Uyi,
    /// Uruguayan Peso
    Uyu,
    /// Uruguayan Unidad Previsional (Pension Unit)
    Uyw,
    /// Uzbekistan Sum
    Uzs,
    /// Venezuala Bolivar
    Veb,
    /// Venezuala Bolivares Fuertes
    Vef,
    /// Venezuela Sovereign Bolivar
    Ves,
    /// Vietnamese Dong
    Vnd,
    /// Vanuatu Vatu
    Vuv,
    /// Samoan Tala
    Wst,
    /// CFA Franc (BEAC)
    Xaf,
    /// Caribbean Dollar
    Xcd,
    /// International Monetary Fund
    Xdr,
    /// UIC-Franc
    Xfu,
    /// CFA Franc (BCEAO)
    Xof,
    /// CFP Franc
    Xpf,
    /// Codes for testing purposes
    Xts,
    /// Codes for transactions/no currencies involved
    Xxx,
    /// North Yemen Rial
    Yer,
    /// Yugoslavian Dinar
    Yum,
    /// South African Cents
    Zac,
    /// South African Rand
    Zar,
    /// Zambian Kwacha
    Zmk,
    /// Zambian New Kwacha
    Zmw,
    /// New Zaire
    Zrn,
    /// Zimbabwe Dollar
    Zwd,
    /// Zimbabwe Gold
    Zwg,
    /// Zimbabwean Dollar
    Zwl,
}
impl Currency {
    /// Converts a Currency to its `str` code.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Adp => "ADP",
            Self::Aed => "AED",
            Self::Afn => "AFN",
            Self::All => "ALL",
            Self::Amd => "AMD",
            Self::Ang => "ANG",
            Self::Aoa => "AOA",
            Self::Ars => "ARS",
            Self::Ats => "ATS",
            Self::Aud => "AUD",
            Self::Awg => "AWG",
            Self::Azn => "AZN",
            Self::Bam => "BAM",
            Self::Bbd => "BBD",
            Self::Bdt => "BDT",
            Self::Bef => "BEF",
            Self::Bgn => "BGN",
            Self::Bhd => "BHD",
            Self::Bif => "BIF",
            Self::Bmd => "BMD",
            Self::Bnd => "BND",
            Self::Bob => "BOB",
            Self::Bov => "BOV",
            Self::Brl => "BRL",
            Self::Bsd => "BSD",
            Self::Btn => "BTN",
            Self::Bwp => "BWP",
            Self::Byn => "BYN",
            Self::Byr => "BYR",
            Self::Bzd => "BZD",
            Self::Cad => "CAD",
            Self::Cdf => "CDF",
            Self::Chf => "CHF",
            Self::Clf => "CLF",
            Self::Clp => "CLP",
            Self::Cny => "CNY",
            Self::Cop => "COP",
            Self::Cou => "COU",
            Self::Crc => "CRC",
            Self::Cup => "CUP",
            Self::Cve => "CVE",
            Self::Cyp => "CYP",
            Self::Czk => "CZK",
            Self::Dem => "DEM",
            Self::Djf => "DJF",
            Self::Dkk => "DKK",
            Self::Dop => "DOP",
            Self::Dzd => "DZD",
            Self::Ecs => "ECS",
            Self::Eek => "EEK",
            Self::Egp => "EGP",
            Self::Ern => "ERN",
            Self::Esp => "ESP",
            Self::Etb => "ETB",
            Self::Eur => "EUR",
            Self::Fim => "FIM",
            Self::Fjd => "FJD",
            Self::Fkp => "FKP",
            Self::Frf => "FRF",
            Self::Gbp => "GBP",
            Self::Gbx => "GBX",
            Self::Gel => "GEL",
            Self::Ghc => "GHC",
            Self::Ghs => "GHS",
            Self::Gip => "GIP",
            Self::Gmd => "GMD",
            Self::Gnf => "GNF",
            Self::Grd => "GRD",
            Self::Gtq => "GTQ",
            Self::Gyd => "GYD",
            Self::Hkd => "HKD",
            Self::Hnl => "HNL",
            Self::Hrk => "HRK",
            Self::Htg => "HTG",
            Self::Huf => "HUF",
            Self::Idr => "IDR",
            Self::Iep => "IEP",
            Self::Ils => "ILS",
            Self::Inr => "INR",
            Self::Iqd => "IQD",
            Self::Irr => "IRR",
            Self::Isk => "ISK",
            Self::Itl => "ITL",
            Self::Jep => "JEP",
            Self::Jmd => "JMD",
            Self::Jod => "JOD",
            Self::Jpy => "JPY",
            Self::Kes => "KES",
            Self::Kgs => "KGS",
            Self::Khr => "KHR",
            Self::Kmf => "KMF",
            Self::Kpw => "KPW",
            Self::Krw => "KRW",
            Self::Kwd => "KWD",
            Self::Kyd => "KYD",
            Self::Kzt => "KZT",
            Self::Lak => "LAK",
            Self::Lbp => "LBP",
            Self::Lkr => "LKR",
            Self::Lrd => "LRD",
            Self::Lsl => "LSL",
            Self::Ltl => "LTL",
            Self::Luf => "LUF",
            Self::Lvl => "LVL",
            Self::Lyd => "LYD",
            Self::Mad => "MAD",
            Self::Mdl => "MDL",
            Self::Mga => "MGA",
            Self::Mkd => "MKD",
            Self::Mmk => "MMK",
            Self::Mnt => "MNT",
            Self::Mop => "MOP",
            Self::Mro => "MRO",
            Self::Mtl => "MTL",
            Self::Mur => "MUR",
            Self::Mvr => "MVR",
            Self::Mwk => "MWK",
            Self::Mxn => "MXN",
            Self::Mxv => "MXV",
            Self::Myr => "MYR",
            Self::Mzn => "MZN",
            Self::Nad => "NAD",
            Self::Ngn => "NGN",
            Self::Nio => "NIO",
            Self::Nlg => "NLG",
            Self::Nok => "NOK",
            Self::Npr => "NPR",
            Self::Nzd => "NZD",
            Self::Omr => "OMR",
            Self::Pab => "PAB",
            Self::Pen => "PEN",
            Self::Pgk => "PGK",
            Self::Php => "PHP",
            Self::Pkr => "PKR",
            Self::Pln => "PLN",
            Self::Pte => "PTE",
            Self::Pyg => "PYG",
            Self::Qar => "QAR",
            Self::Rol => "ROL",
            Self::Ron => "RON",
            Self::Rsd => "RSD",
            Self::Rub => "RUB",
            Self::Rur => "RUR",
            Self::Rwf => "RWF",
            Self::Sar => "SAR",
            Self::Sbd => "SBD",
            Self::Scr => "SCR",
            Self::Sdd => "SDD",
            Self::Sdg => "SDG",
            Self::Sek => "SEK",
            Self::Sgd => "SGD",
            Self::Shp => "SHP",
            Self::Sit => "SIT",
            Self::Skk => "SKK",
            Self::Sll => "SLL",
            Self::Sos => "SOS",
            Self::Srd => "SRD",
            Self::Std => "STD",
            Self::Svc => "SVC",
            Self::Syp => "SYP",
            Self::Szl => "SZL",
            Self::Thb => "THB",
            Self::Tjs => "TJS",
            Self::Tmm => "TMM",
            Self::Tnd => "TND",
            Self::Top => "TOP",
            Self::Trl => "TRL",
            Self::Try => "TRY",
            Self::Ttd => "TTD",
            Self::Twd => "TWD",
            Self::Tzs => "TZS",
            Self::Uah => "UAH",
            Self::Ugx => "UGX",
            Self::Usd => "USD",
            Self::Usx => "USX",
            Self::Uyi => "UYI",
            Self::Uyu => "UYU",
            Self::Uyw => "UYW",
            Self::Uzs => "UZS",
            Self::Veb => "VEB",
            Self::Vef => "VEF",
            Self::Ves => "VES",
            Self::Vnd => "VND",
            Self::Vuv => "VUV",
            Self::Wst => "WST",
            Self::Xaf => "XAF",
            Self::Xcd => "XCD",
            Self::Xdr => "XDR",
            Self::Xfu => "XFU",
            Self::Xof => "XOF",
            Self::Xpf => "XPF",
            Self::Xts => "XTS",
            Self::Xxx => "XXX",
            Self::Yer => "YER",
            Self::Yum => "YUM",
            Self::Zac => "ZAC",
            Self::Zar => "ZAR",
            Self::Zmk => "ZMK",
            Self::Zmw => "ZMW",
            Self::Zrn => "ZRN",
            Self::Zwd => "ZWD",
            Self::Zwg => "ZWG",
            Self::Zwl => "ZWL",
        }
    }
}

impl AsRef<str> for Currency {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::str::FromStr for Currency {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "ADP" => Ok(Self::Adp),
            "AED" => Ok(Self::Aed),
            "AFN" => Ok(Self::Afn),
            "ALL" => Ok(Self::All),
            "AMD" => Ok(Self::Amd),
            "ANG" => Ok(Self::Ang),
            "AOA" => Ok(Self::Aoa),
            "ARS" => Ok(Self::Ars),
            "ATS" => Ok(Self::Ats),
            "AUD" => Ok(Self::Aud),
            "AWG" => Ok(Self::Awg),
            "AZN" => Ok(Self::Azn),
            "BAM" => Ok(Self::Bam),
            "BBD" => Ok(Self::Bbd),
            "BDT" => Ok(Self::Bdt),
            "BEF" => Ok(Self::Bef),
            "BGN" => Ok(Self::Bgn),
            "BHD" => Ok(Self::Bhd),
            "BIF" => Ok(Self::Bif),
            "BMD" => Ok(Self::Bmd),
            "BND" => Ok(Self::Bnd),
            "BOB" => Ok(Self::Bob),
            "BOV" => Ok(Self::Bov),
            "BRL" => Ok(Self::Brl),
            "BSD" => Ok(Self::Bsd),
            "BTN" => Ok(Self::Btn),
            "BWP" => Ok(Self::Bwp),
            "BYN" => Ok(Self::Byn),
            "BYR" => Ok(Self::Byr),
            "BZD" => Ok(Self::Bzd),
            "CAD" => Ok(Self::Cad),
            "CDF" => Ok(Self::Cdf),
            "CHF" => Ok(Self::Chf),
            "CLF" => Ok(Self::Clf),
            "CLP" => Ok(Self::Clp),
            "CNY" => Ok(Self::Cny),
            "COP" => Ok(Self::Cop),
            "COU" => Ok(Self::Cou),
            "CRC" => Ok(Self::Crc),
            "CUP" => Ok(Self::Cup),
            "CVE" => Ok(Self::Cve),
            "CYP" => Ok(Self::Cyp),
            "CZK" => Ok(Self::Czk),
            "DEM" => Ok(Self::Dem),
            "DJF" => Ok(Self::Djf),
            "DKK" => Ok(Self::Dkk),
            "DOP" => Ok(Self::Dop),
            "DZD" => Ok(Self::Dzd),
            "ECS" => Ok(Self::Ecs),
            "EEK" => Ok(Self::Eek),
            "EGP" => Ok(Self::Egp),
            "ERN" => Ok(Self::Ern),
            "ESP" => Ok(Self::Esp),
            "ETB" => Ok(Self::Etb),
            "EUR" => Ok(Self::Eur),
            "FIM" => Ok(Self::Fim),
            "FJD" => Ok(Self::Fjd),
            "FKP" => Ok(Self::Fkp),
            "FRF" => Ok(Self::Frf),
            "GBP" => Ok(Self::Gbp),
            "GBX" => Ok(Self::Gbx),
            "GEL" => Ok(Self::Gel),
            "GHC" => Ok(Self::Ghc),
            "GHS" => Ok(Self::Ghs),
            "GIP" => Ok(Self::Gip),
            "GMD" => Ok(Self::Gmd),
            "GNF" => Ok(Self::Gnf),
            "GRD" => Ok(Self::Grd),
            "GTQ" => Ok(Self::Gtq),
            "GYD" => Ok(Self::Gyd),
            "HKD" => Ok(Self::Hkd),
            "HNL" => Ok(Self::Hnl),
            "HRK" => Ok(Self::Hrk),
            "HTG" => Ok(Self::Htg),
            "HUF" => Ok(Self::Huf),
            "IDR" => Ok(Self::Idr),
            "IEP" => Ok(Self::Iep),
            "ILS" => Ok(Self::Ils),
            "INR" => Ok(Self::Inr),
            "IQD" => Ok(Self::Iqd),
            "IRR" => Ok(Self::Irr),
            "ISK" => Ok(Self::Isk),
            "ITL" => Ok(Self::Itl),
            "JEP" => Ok(Self::Jep),
            "JMD" => Ok(Self::Jmd),
            "JOD" => Ok(Self::Jod),
            "JPY" => Ok(Self::Jpy),
            "KES" => Ok(Self::Kes),
            "KGS" => Ok(Self::Kgs),
            "KHR" => Ok(Self::Khr),
            "KMF" => Ok(Self::Kmf),
            "KPW" => Ok(Self::Kpw),
            "KRW" => Ok(Self::Krw),
            "KWD" => Ok(Self::Kwd),
            "KYD" => Ok(Self::Kyd),
            "KZT" => Ok(Self::Kzt),
            "LAK" => Ok(Self::Lak),
            "LBP" => Ok(Self::Lbp),
            "LKR" => Ok(Self::Lkr),
            "LRD" => Ok(Self::Lrd),
            "LSL" => Ok(Self::Lsl),
            "LTL" => Ok(Self::Ltl),
            "LUF" => Ok(Self::Luf),
            "LVL" => Ok(Self::Lvl),
            "LYD" => Ok(Self::Lyd),
            "MAD" => Ok(Self::Mad),
            "MDL" => Ok(Self::Mdl),
            "MGA" => Ok(Self::Mga),
            "MKD" => Ok(Self::Mkd),
            "MMK" => Ok(Self::Mmk),
            "MNT" => Ok(Self::Mnt),
            "MOP" => Ok(Self::Mop),
            "MRO" => Ok(Self::Mro),
            "MTL" => Ok(Self::Mtl),
            "MUR" => Ok(Self::Mur),
            "MVR" => Ok(Self::Mvr),
            "MWK" => Ok(Self::Mwk),
            "MXN" => Ok(Self::Mxn),
            "MXV" => Ok(Self::Mxv),
            "MYR" => Ok(Self::Myr),
            "MZN" => Ok(Self::Mzn),
            "NAD" => Ok(Self::Nad),
            "NGN" => Ok(Self::Ngn),
            "NIO" => Ok(Self::Nio),
            "NLG" => Ok(Self::Nlg),
            "NOK" => Ok(Self::Nok),
            "NPR" => Ok(Self::Npr),
            "NZD" => Ok(Self::Nzd),
            "OMR" => Ok(Self::Omr),
            "PAB" => Ok(Self::Pab),
            "PEN" => Ok(Self::Pen),
            "PGK" => Ok(Self::Pgk),
            "PHP" => Ok(Self::Php),
            "PKR" => Ok(Self::Pkr),
            "PLN" => Ok(Self::Pln),
            "PTE" => Ok(Self::Pte),
            "PYG" => Ok(Self::Pyg),
            "QAR" => Ok(Self::Qar),
            "ROL" => Ok(Self::Rol),
            "RON" => Ok(Self::Ron),
            "RSD" => Ok(Self::Rsd),
            "RUB" => Ok(Self::Rub),
            "RUR" => Ok(Self::Rur),
            "RWF" => Ok(Self::Rwf),
            "SAR" => Ok(Self::Sar),
            "SBD" => Ok(Self::Sbd),
            "SCR" => Ok(Self::Scr),
            "SDD" => Ok(Self::Sdd),
            "SDG" => Ok(Self::Sdg),
            "SEK" => Ok(Self::Sek),
            "SGD" => Ok(Self::Sgd),
            "SHP" => Ok(Self::Shp),
            "SIT" => Ok(Self::Sit),
            "SKK" => Ok(Self::Skk),
            "SLL" => Ok(Self::Sll),
            "SOS" => Ok(Self::Sos),
            "SRD" => Ok(Self::Srd),
            "STD" => Ok(Self::Std),
            "SVC" => Ok(Self::Svc),
            "SYP" => Ok(Self::Syp),
            "SZL" => Ok(Self::Szl),
            "THB" => Ok(Self::Thb),
            "TJS" => Ok(Self::Tjs),
            "TMM" => Ok(Self::Tmm),
            "TND" => Ok(Self::Tnd),
            "TOP" => Ok(Self::Top),
            "TRL" => Ok(Self::Trl),
            "TRY" => Ok(Self::Try),
            "TTD" => Ok(Self::Ttd),
            "TWD" => Ok(Self::Twd),
            "TZS" => Ok(Self::Tzs),
            "UAH" => Ok(Self::Uah),
            "UGX" => Ok(Self::Ugx),
            "USD" => Ok(Self::Usd),
            "USX" => Ok(Self::Usx),
            "UYI" => Ok(Self::Uyi),
            "UYU" => Ok(Self::Uyu),
            "UYW" => Ok(Self::Uyw),
            "UZS" => Ok(Self::Uzs),
            "VEB" => Ok(Self::Veb),
            "VEF" => Ok(Self::Vef),
            "VES" => Ok(Self::Ves),
            "VND" => Ok(Self::Vnd),
            "VUV" => Ok(Self::Vuv),
            "WST" => Ok(Self::Wst),
            "XAF" => Ok(Self::Xaf),
            "XCD" => Ok(Self::Xcd),
            "XDR" => Ok(Self::Xdr),
            "XFU" => Ok(Self::Xfu),
            "XOF" => Ok(Self::Xof),
            "XPF" => Ok(Self::Xpf),
            "XTS" => Ok(Self::Xts),
            "XXX" => Ok(Self::Xxx),
            "YER" => Ok(Self::Yer),
            "YUM" => Ok(Self::Yum),
            "ZAC" => Ok(Self::Zac),
            "ZAR" => Ok(Self::Zar),
            "ZMK" => Ok(Self::Zmk),
            "ZMW" => Ok(Self::Zmw),
            "ZRN" => Ok(Self::Zrn),
            "ZWD" => Ok(Self::Zwd),
            "ZWG" => Ok(Self::Zwg),
            "ZWL" => Ok(Self::Zwl),
            _ => Err(Error::bad_arg(
                "s",
                format!("no Currency variant associated with {s}"),
            )),
        }
    }
}

impl<'de> Deserialize<'de> for Currency {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let str = String::deserialize(deserializer)?;
        FromStr::from_str(&str).map_err(de::Error::custom)
    }
}

impl Serialize for Currency {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_str().serialize(serializer)
    }
}

/// A corporate actions event type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Event {
    /// Company Meeting
    Agm,
    /// Announcement
    Ann,
    /// Arrangement
    Arr,
    /// Assimilation
    Assm,
    /// Buyback
    Bb,
    /// Bloomberg Composite ID Change
    Bbcc,
    /// Bloomberg Exchange ID Change
    Bbec,
    /// Bankruptcy
    Bkrp,
    /// Bonus Issue
    Bon,
    /// Bonus Rights
    Br,
    /// Call
    Call,
    /// Capital Reduction
    Caprd,
    /// Class Action
    Clsac,
    /// Consolidation
    Consd,
    /// Conversion
    Conv,
    /// Certificate of Exchange
    Ctx,
    /// Currency Redenomination
    Currd,
    /// Distribution
    Dist,
    /// Dividend
    Div,
    /// Dividend - Equity Bifurcated
    Diveb,
    /// Dividend Reclassification
    Divrc,
    /// Demerger
    Dmrgr,
    /// Depository Receipt Change
    Drchg,
    /// Dividend Reinvestment Plan
    Drip,
    /// Divestment
    Dvst,
    /// Entitlement Issue
    Ent,
    /// Franking
    Frank,
    /// Forward Split
    Fsplt,
    /// Financial Transaction Tax
    Ftt,
    /// Financial Year Change
    Fychg,
    /// International Code Change
    Icc,
    /// Country of Incorporation Change
    Inchg,
    /// Issuer Name Change
    Ischg,
    /// Local Code Change
    Lcc,
    /// Liquidation
    Liq,
    /// Listing Status
    Lstat,
    /// Lot Change
    Ltchg,
    /// Market Segment Change
    Mkchg,
    /// Merger
    Mrgr,
    /// New Listing
    Nlist,
    /// Odd Lot Offer
    Oddlt,
    /// Property Income Distribution
    Pid,
    /// Purchase Offer
    Po,
    /// Primary Exchange Change
    Prchg,
    /// Preferential Offer
    Prf,
    /// Par Value Redenomination
    Pvrd,
    /// Return of Capital
    Rcap,
    /// Record Date
    Rd,
    /// Redemption
    Redem,
    /// Reverse Split
    Rsplt,
    /// Rights
    Rts,
    /// Security Name Change
    Scchg,
    /// Security Swap
    Scswp,
    /// Sub Division
    Sd,
    /// Sedol Change
    Sdchg,
    /// Security Re-classification
    Secrc,
    /// Shares Outstanding Change
    Shoch,
    /// Spin-Off
    Soff,
    /// Takeover
    Tkovr,
    /// Warrant Exercise
    Warex,
}
impl Event {
    /// Converts a Event to its `str` code.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Agm => "AGM",
            Self::Ann => "ANN",
            Self::Arr => "ARR",
            Self::Assm => "ASSM",
            Self::Bb => "BB",
            Self::Bbcc => "BBCC",
            Self::Bbec => "BBEC",
            Self::Bkrp => "BKRP",
            Self::Bon => "BON",
            Self::Br => "BR",
            Self::Call => "CALL",
            Self::Caprd => "CAPRD",
            Self::Clsac => "CLSAC",
            Self::Consd => "CONSD",
            Self::Conv => "CONV",
            Self::Ctx => "CTX",
            Self::Currd => "CURRD",
            Self::Dist => "DIST",
            Self::Div => "DIV",
            Self::Diveb => "DIVEB",
            Self::Divrc => "DIVRC",
            Self::Dmrgr => "DMRGR",
            Self::Drchg => "DRCHG",
            Self::Drip => "DRIP",
            Self::Dvst => "DVST",
            Self::Ent => "ENT",
            Self::Frank => "FRANK",
            Self::Fsplt => "FSPLT",
            Self::Ftt => "FTT",
            Self::Fychg => "FYCHG",
            Self::Icc => "ICC",
            Self::Inchg => "INCHG",
            Self::Ischg => "ISCHG",
            Self::Lcc => "LCC",
            Self::Liq => "LIQ",
            Self::Lstat => "LSTAT",
            Self::Ltchg => "LTCHG",
            Self::Mkchg => "MKCHG",
            Self::Mrgr => "MRGR",
            Self::Nlist => "NLIST",
            Self::Oddlt => "ODDLT",
            Self::Pid => "PID",
            Self::Po => "PO",
            Self::Prchg => "PRCHG",
            Self::Prf => "PRF",
            Self::Pvrd => "PVRD",
            Self::Rcap => "RCAP",
            Self::Rd => "RD",
            Self::Redem => "REDEM",
            Self::Rsplt => "RSPLT",
            Self::Rts => "RTS",
            Self::Scchg => "SCCHG",
            Self::Scswp => "SCSWP",
            Self::Sd => "SD",
            Self::Sdchg => "SDCHG",
            Self::Secrc => "SECRC",
            Self::Shoch => "SHOCH",
            Self::Soff => "SOFF",
            Self::Tkovr => "TKOVR",
            Self::Warex => "WAREX",
        }
    }
}

impl AsRef<str> for Event {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::str::FromStr for Event {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "AGM" => Ok(Self::Agm),
            "ANN" => Ok(Self::Ann),
            "ARR" => Ok(Self::Arr),
            "ASSM" => Ok(Self::Assm),
            "BB" => Ok(Self::Bb),
            "BBCC" => Ok(Self::Bbcc),
            "BBEC" => Ok(Self::Bbec),
            "BKRP" => Ok(Self::Bkrp),
            "BON" => Ok(Self::Bon),
            "BR" => Ok(Self::Br),
            "CALL" => Ok(Self::Call),
            "CAPRD" => Ok(Self::Caprd),
            "CLSAC" => Ok(Self::Clsac),
            "CONSD" => Ok(Self::Consd),
            "CONV" => Ok(Self::Conv),
            "CTX" => Ok(Self::Ctx),
            "CURRD" => Ok(Self::Currd),
            "DIST" => Ok(Self::Dist),
            "DIV" => Ok(Self::Div),
            "DIVEB" => Ok(Self::Diveb),
            "DIVRC" => Ok(Self::Divrc),
            "DMRGR" => Ok(Self::Dmrgr),
            "DRCHG" => Ok(Self::Drchg),
            "DRIP" => Ok(Self::Drip),
            "DVST" => Ok(Self::Dvst),
            "ENT" => Ok(Self::Ent),
            "FRANK" => Ok(Self::Frank),
            "FSPLT" => Ok(Self::Fsplt),
            "FTT" => Ok(Self::Ftt),
            "FYCHG" => Ok(Self::Fychg),
            "ICC" => Ok(Self::Icc),
            "INCHG" => Ok(Self::Inchg),
            "ISCHG" => Ok(Self::Ischg),
            "LCC" => Ok(Self::Lcc),
            "LIQ" => Ok(Self::Liq),
            "LSTAT" => Ok(Self::Lstat),
            "LTCHG" => Ok(Self::Ltchg),
            "MKCHG" => Ok(Self::Mkchg),
            "MRGR" => Ok(Self::Mrgr),
            "NLIST" => Ok(Self::Nlist),
            "ODDLT" => Ok(Self::Oddlt),
            "PID" => Ok(Self::Pid),
            "PO" => Ok(Self::Po),
            "PRCHG" => Ok(Self::Prchg),
            "PRF" => Ok(Self::Prf),
            "PVRD" => Ok(Self::Pvrd),
            "RCAP" => Ok(Self::Rcap),
            "RD" => Ok(Self::Rd),
            "REDEM" => Ok(Self::Redem),
            "RSPLT" => Ok(Self::Rsplt),
            "RTS" => Ok(Self::Rts),
            "SCCHG" => Ok(Self::Scchg),
            "SCSWP" => Ok(Self::Scswp),
            "SD" => Ok(Self::Sd),
            "SDCHG" => Ok(Self::Sdchg),
            "SECRC" => Ok(Self::Secrc),
            "SHOCH" => Ok(Self::Shoch),
            "SOFF" => Ok(Self::Soff),
            "TKOVR" => Ok(Self::Tkovr),
            "WAREX" => Ok(Self::Warex),
            _ => Err(Error::bad_arg(
                "s",
                format!("no Event variant associated with {s}"),
            )),
        }
    }
}

impl<'de> Deserialize<'de> for Event {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let str = String::deserialize(deserializer)?;
        FromStr::from_str(&str).map_err(de::Error::custom)
    }
}

impl Serialize for Event {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_str().serialize(serializer)
    }
}

/// A corporate actions sub-event type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EventSubType {
    /// Annual General Meeting
    Agm,
    /// Bond Holder Meeting
    Bhm,
    /// Court Ordered General Meeting
    Cgm,
    /// Extraordinary General Meeting
    Egm,
    /// General Meeting
    Gm,
    /// Special General Meeting
    Sgm,
    /// Dutch Auction
    Dutchauct,
    /// Depository Receipt Dividend
    Dprcpdiv,
    /// Reverse Split
    Consd,
    /// Buyback
    Bb,
    /// Call Option Exercised
    Call,
    /// Drawings by lottery
    Drl,
    /// Early Conversion
    Econv,
    /// Maturity
    Mat,
    /// Ordinary
    Ord,
    /// Put Option Exercised
    Put,
    /// Spin-Off
    Dist,
    /// Capital Distribution
    Capdist,
    /// Forward Split
    Div,
    /// Derived from Interest Payment
    Intdiv,
    /// Interest Basis Unknown
    Int,
    /// Spin-Off
    Dmrgr,
    /// Non Renounceable Rights
    Nrenrts,
    /// Open Offer
    Opoff,
    /// Priority Offer
    Poff,
    /// Share Purchase Plan
    Spp,
    /// Fully franked
    F,
    /// Partially franked
    P,
    /// Unfranked
    U,
    /// Capital Reduction
    Caprd,
    /// Liquidation
    Liq,
    /// Reserves
    Res,
    /// Sale of Assets
    Soa,
    /// Share Premium Account
    Spa,
    /// Amortisation
    Amt,
    /// Buyback Early Deadline
    Bbed,
    /// Buyback Regular Deadline
    Bbrd,
    /// Early Redemption
    Er,
    /// Make Whole Call
    Mwc,
    /// Forward Split
    Sd,
    /// Merger
    Mrgr,
    /// Tender Offer
    Tend,
    /// Tender resulting in Merger
    Tendmrgr,
    /// Mini-Takeover
    Tkovrmini,
    /// Insufficient data to assign a TKOVR event subtype
    Ukwnsubtyp,
}
impl EventSubType {
    /// Converts a EventSubType to its `str` code.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Agm => "AGM",
            Self::Bhm => "BHM",
            Self::Cgm => "CGM",
            Self::Egm => "EGM",
            Self::Gm => "GM",
            Self::Sgm => "SGM",
            Self::Dutchauct => "DUTCHAUCT",
            Self::Dprcpdiv => "DPRCPDIV",
            Self::Consd => "CONSD",
            Self::Bb => "BB",
            Self::Call => "CALL",
            Self::Drl => "DRL",
            Self::Econv => "ECONV",
            Self::Mat => "MAT",
            Self::Ord => "ORD",
            Self::Put => "PUT",
            Self::Dist => "DIST",
            Self::Capdist => "CAPDIST",
            Self::Div => "DIV",
            Self::Intdiv => "INTDIV",
            Self::Int => "INT",
            Self::Dmrgr => "DMRGR",
            Self::Nrenrts => "NRENRTS",
            Self::Opoff => "OPOFF",
            Self::Poff => "POFF",
            Self::Spp => "SPP",
            Self::F => "F",
            Self::P => "P",
            Self::U => "U",
            Self::Caprd => "CAPRD",
            Self::Liq => "LIQ",
            Self::Res => "RES",
            Self::Soa => "SOA",
            Self::Spa => "SPA",
            Self::Amt => "AMT",
            Self::Bbed => "BBED",
            Self::Bbrd => "BBRD",
            Self::Er => "ER",
            Self::Mwc => "MWC",
            Self::Sd => "SD",
            Self::Mrgr => "MRGR",
            Self::Tend => "TEND",
            Self::Tendmrgr => "TENDMRGR",
            Self::Tkovrmini => "TKOVRMINI",
            Self::Ukwnsubtyp => "UKWNSUBTYP",
        }
    }
}

impl AsRef<str> for EventSubType {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::str::FromStr for EventSubType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "AGM" => Ok(Self::Agm),
            "BHM" => Ok(Self::Bhm),
            "CGM" => Ok(Self::Cgm),
            "EGM" => Ok(Self::Egm),
            "GM" => Ok(Self::Gm),
            "SGM" => Ok(Self::Sgm),
            "DUTCHAUCT" => Ok(Self::Dutchauct),
            "DPRCPDIV" => Ok(Self::Dprcpdiv),
            "CONSD" => Ok(Self::Consd),
            "BB" => Ok(Self::Bb),
            "CALL" => Ok(Self::Call),
            "DRL" => Ok(Self::Drl),
            "ECONV" => Ok(Self::Econv),
            "MAT" => Ok(Self::Mat),
            "ORD" => Ok(Self::Ord),
            "PUT" => Ok(Self::Put),
            "DIST" => Ok(Self::Dist),
            "CAPDIST" => Ok(Self::Capdist),
            "DIV" => Ok(Self::Div),
            "INTDIV" => Ok(Self::Intdiv),
            "INT" => Ok(Self::Int),
            "DMRGR" => Ok(Self::Dmrgr),
            "NRENRTS" => Ok(Self::Nrenrts),
            "OPOFF" => Ok(Self::Opoff),
            "POFF" => Ok(Self::Poff),
            "SPP" => Ok(Self::Spp),
            "F" => Ok(Self::F),
            "P" => Ok(Self::P),
            "U" => Ok(Self::U),
            "CAPRD" => Ok(Self::Caprd),
            "LIQ" => Ok(Self::Liq),
            "RES" => Ok(Self::Res),
            "SOA" => Ok(Self::Soa),
            "SPA" => Ok(Self::Spa),
            "AMT" => Ok(Self::Amt),
            "BBED" => Ok(Self::Bbed),
            "BBRD" => Ok(Self::Bbrd),
            "ER" => Ok(Self::Er),
            "MWC" => Ok(Self::Mwc),
            "SD" => Ok(Self::Sd),
            "MRGR" => Ok(Self::Mrgr),
            "TEND" => Ok(Self::Tend),
            "TENDMRGR" => Ok(Self::Tendmrgr),
            "TKOVRMINI" => Ok(Self::Tkovrmini),
            "UKWNSUBTYP" => Ok(Self::Ukwnsubtyp),
            _ => Err(Error::bad_arg(
                "s",
                format!("no EventSubType variant associated with {s}"),
            )),
        }
    }
}

impl<'de> Deserialize<'de> for EventSubType {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let str = String::deserialize(deserializer)?;
        FromStr::from_str(&str).map_err(de::Error::custom)
    }
}

impl Serialize for EventSubType {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_str().serialize(serializer)
    }
}

/// How fractions are handled at settlement.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Fraction {
    /// Cash
    Cash = b'C',
    /// Round Down
    RoundDown = b'D',
    /// Fractions
    Fractions = b'F',
    /// Round Up
    RoundUp = b'U',
}
impl From<Fraction> for u8 {
    fn from(value: Fraction) -> u8 {
        value as u8
    }
}

impl From<Fraction> for char {
    fn from(value: Fraction) -> char {
        u8::from(value) as char
    }
}

impl TryFrom<u8> for Fraction {
    type Error = Error;

    fn try_from(value: u8) -> std::result::Result<Self, Error> {
        match value {
            b'C' => Ok(Self::Cash),
            b'D' => Ok(Self::RoundDown),
            b'F' => Ok(Self::Fractions),
            b'U' => Ok(Self::RoundUp),
            _ => Err(Error::bad_arg(
                "value",
                format!("no Fraction variant associated with {value:?}"),
            )),
        }
    }
}

impl<'de> Deserialize<'de> for Fraction {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let c = char::deserialize(deserializer)?;
        let u = u8::try_from(c).map_err(de::Error::custom)?;
        Fraction::try_from(u).map_err(de::Error::custom)
    }
}

impl Serialize for Fraction {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        (*self as u8 as char).serialize(serializer)
    }
}

/// The dividend frequency.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Frequency {
    /// 28 Days
    Days28,
    /// Every 35 Days
    Days35,
    /// Annual
    Annual,
    /// BiMonthly
    BiMonthly,
    /// Daily
    Daily,
    /// Final
    Final,
    /// Interim
    Interim,
    /// Interest on Maturity
    Intonmat,
    /// Interest on Trigger
    Intontrig,
    /// Irregular
    Irregular,
    /// Interest on Maturity
    Itm,
    /// Monthly
    Monthly,
    /// Quarterly
    Quarterly,
    /// Semi-Annual
    SemiAnnual,
    /// Trimesterly
    Trimesterly,
    /// Unspecified
    Unspecified,
    /// Weekly
    Weekly,
}
impl Frequency {
    /// Converts a Frequency to its `str` code.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Days28 => "28D",
            Self::Days35 => "35DAY",
            Self::Annual => "ANL",
            Self::BiMonthly => "BIM",
            Self::Daily => "DLY",
            Self::Final => "FNL",
            Self::Interim => "INT",
            Self::Intonmat => "INTONMAT",
            Self::Intontrig => "INTONTRIG",
            Self::Irregular => "IRG",
            Self::Itm => "ITM",
            Self::Monthly => "MNT",
            Self::Quarterly => "QTR",
            Self::SemiAnnual => "SMA",
            Self::Trimesterly => "TRM",
            Self::Unspecified => "UN",
            Self::Weekly => "WKL",
        }
    }
}

impl AsRef<str> for Frequency {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::str::FromStr for Frequency {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "28D" => Ok(Self::Days28),
            "35DAY" => Ok(Self::Days35),
            "ANL" => Ok(Self::Annual),
            "BIM" => Ok(Self::BiMonthly),
            "DLY" => Ok(Self::Daily),
            "FNL" => Ok(Self::Final),
            "INT" => Ok(Self::Interim),
            "INTONMAT" => Ok(Self::Intonmat),
            "INTONTRIG" => Ok(Self::Intontrig),
            "IRG" => Ok(Self::Irregular),
            "ITM" => Ok(Self::Itm),
            "MNT" => Ok(Self::Monthly),
            "QTR" => Ok(Self::Quarterly),
            "SMA" => Ok(Self::SemiAnnual),
            "TRM" => Ok(Self::Trimesterly),
            "UN" => Ok(Self::Unspecified),
            "WKL" => Ok(Self::Weekly),
            _ => Err(Error::bad_arg(
                "s",
                format!("no Frequency variant associated with {s}"),
            )),
        }
    }
}

impl<'de> Deserialize<'de> for Frequency {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let str = String::deserialize(deserializer)?;
        FromStr::from_str(&str).map_err(de::Error::custom)
    }
}

impl Serialize for Frequency {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_str().serialize(serializer)
    }
}

/// The global status code. Indicates the global listing activity status of a security.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum GlobalStatus {
    /// Active
    Active = b'A',
    /// In default
    InDefault = b'D',
    /// Inactive
    Inactive = b'I',
}
impl From<GlobalStatus> for u8 {
    fn from(value: GlobalStatus) -> u8 {
        value as u8
    }
}

impl From<GlobalStatus> for char {
    fn from(value: GlobalStatus) -> char {
        u8::from(value) as char
    }
}

impl TryFrom<u8> for GlobalStatus {
    type Error = Error;

    fn try_from(value: u8) -> std::result::Result<Self, Error> {
        match value {
            b'A' => Ok(Self::Active),
            b'D' => Ok(Self::InDefault),
            b'I' => Ok(Self::Inactive),
            _ => Err(Error::bad_arg(
                "value",
                format!("no GlobalStatus variant associated with {value:?}"),
            )),
        }
    }
}

impl<'de> Deserialize<'de> for GlobalStatus {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let c = char::deserialize(deserializer)?;
        let u = u8::try_from(c).map_err(de::Error::custom)?;
        GlobalStatus::try_from(u).map_err(de::Error::custom)
    }
}

impl Serialize for GlobalStatus {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        (*self as u8 as char).serialize(serializer)
    }
}

/// Listing source.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum ListingSource {
    /// Main WCA supported Listing
    Main = b'M',
    /// Secondary Listing
    Secondary = b'S',
}
impl From<ListingSource> for u8 {
    fn from(value: ListingSource) -> u8 {
        value as u8
    }
}

impl From<ListingSource> for char {
    fn from(value: ListingSource) -> char {
        u8::from(value) as char
    }
}

impl TryFrom<u8> for ListingSource {
    type Error = Error;

    fn try_from(value: u8) -> std::result::Result<Self, Error> {
        match value {
            b'M' => Ok(Self::Main),
            b'S' => Ok(Self::Secondary),
            _ => Err(Error::bad_arg(
                "value",
                format!("no ListingSource variant associated with {value:?}"),
            )),
        }
    }
}

impl<'de> Deserialize<'de> for ListingSource {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let c = char::deserialize(deserializer)?;
        let u = u8::try_from(c).map_err(de::Error::custom)?;
        ListingSource::try_from(u).map_err(de::Error::custom)
    }
}

impl Serialize for ListingSource {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        (*self as u8 as char).serialize(serializer)
    }
}

/// Listing status.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum ListingStatus {
    /// Delisted
    Delisted = b'D',
    /// Reporting purposes only - Listed
    RpoListed = b'G',
    /// Reporting purposes only - Delisted
    RpoDelisted = b'H',
    /// Reporting purposes only - Suspended
    RpoSuspended = b'I',
    /// Listed
    Listed = b'L',
    /// New Listing
    New = b'N',
    /// Listing Pending
    Pending = b'P',
    /// Resumed
    Resumed = b'R',
    /// Suspended
    Suspended = b'S',
    /// Trading permitted - Listed
    TpListed = b'T',
    /// Trading permitted - Delisted
    TpDelisted = b'U',
    /// Trading permitted - Suspended
    TpSuspended = b'V',
}
impl From<ListingStatus> for u8 {
    fn from(value: ListingStatus) -> u8 {
        value as u8
    }
}

impl From<ListingStatus> for char {
    fn from(value: ListingStatus) -> char {
        u8::from(value) as char
    }
}

impl TryFrom<u8> for ListingStatus {
    type Error = Error;

    fn try_from(value: u8) -> std::result::Result<Self, Error> {
        match value {
            b'D' => Ok(Self::Delisted),
            b'G' => Ok(Self::RpoListed),
            b'H' => Ok(Self::RpoDelisted),
            b'I' => Ok(Self::RpoSuspended),
            b'L' => Ok(Self::Listed),
            b'N' => Ok(Self::New),
            b'P' => Ok(Self::Pending),
            b'R' => Ok(Self::Resumed),
            b'S' => Ok(Self::Suspended),
            b'T' => Ok(Self::TpListed),
            b'U' => Ok(Self::TpDelisted),
            b'V' => Ok(Self::TpSuspended),
            _ => Err(Error::bad_arg(
                "value",
                format!("no ListingStatus variant associated with {value:?}"),
            )),
        }
    }
}

impl<'de> Deserialize<'de> for ListingStatus {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let c = char::deserialize(deserializer)?;
        let u = u8::try_from(c).map_err(de::Error::custom)?;
        ListingStatus::try_from(u).map_err(de::Error::custom)
    }
}

impl Serialize for ListingStatus {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        (*self as u8 as char).serialize(serializer)
    }
}

/// Mandatory or voluntary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum MandVolu {
    /// Mandatory
    Mandatory = b'M',
    /// Voluntary
    Voluntary = b'V',
    /// Mandatory and/or voluntary
    MandVolu = b'W',
}
impl From<MandVolu> for u8 {
    fn from(value: MandVolu) -> u8 {
        value as u8
    }
}

impl From<MandVolu> for char {
    fn from(value: MandVolu) -> char {
        u8::from(value) as char
    }
}

impl TryFrom<u8> for MandVolu {
    type Error = Error;

    fn try_from(value: u8) -> std::result::Result<Self, Error> {
        match value {
            b'M' => Ok(Self::Mandatory),
            b'V' => Ok(Self::Voluntary),
            b'W' => Ok(Self::MandVolu),
            _ => Err(Error::bad_arg(
                "value",
                format!("no MandVolu variant associated with {value:?}"),
            )),
        }
    }
}

impl<'de> Deserialize<'de> for MandVolu {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let c = char::deserialize(deserializer)?;
        let u = u8::try_from(c).map_err(de::Error::custom)?;
        MandVolu::try_from(u).map_err(de::Error::custom)
    }
}

impl Serialize for MandVolu {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        (*self as u8 as char).serialize(serializer)
    }
}

/// The style of outturn security.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OutturnStyle {
    /// Additional for Existing Securities
    Adex,
    /// New for Old Securities
    Newo,
}
impl OutturnStyle {
    /// Converts a OutturnStyle to its `str` code.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Adex => "ADEX",
            Self::Newo => "NEWO",
        }
    }
}

impl AsRef<str> for OutturnStyle {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::str::FromStr for OutturnStyle {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "ADEX" => Ok(Self::Adex),
            "NEWO" => Ok(Self::Newo),
            _ => Err(Error::bad_arg(
                "s",
                format!("no OutturnStyle variant associated with {s}"),
            )),
        }
    }
}

impl<'de> Deserialize<'de> for OutturnStyle {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let str = String::deserialize(deserializer)?;
        FromStr::from_str(&str).map_err(de::Error::custom)
    }
}

impl Serialize for OutturnStyle {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_str().serialize(serializer)
    }
}

/// The payment type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum PaymentType {
    /// Cash&Stock
    CashAndStock = b'B',
    /// Cash
    Cash = b'C',
    /// Dissenters Rights
    DissentersRights = b'D',
    /// Stock
    Stock = b'S',
    /// ToBeAnnounced
    Tba = b'T',
}
impl From<PaymentType> for u8 {
    fn from(value: PaymentType) -> u8 {
        value as u8
    }
}

impl From<PaymentType> for char {
    fn from(value: PaymentType) -> char {
        u8::from(value) as char
    }
}

impl TryFrom<u8> for PaymentType {
    type Error = Error;

    fn try_from(value: u8) -> std::result::Result<Self, Error> {
        match value {
            b'B' => Ok(Self::CashAndStock),
            b'C' => Ok(Self::Cash),
            b'D' => Ok(Self::DissentersRights),
            b'S' => Ok(Self::Stock),
            b'T' => Ok(Self::Tba),
            _ => Err(Error::bad_arg(
                "value",
                format!("no PaymentType variant associated with {value:?}"),
            )),
        }
    }
}

impl<'de> Deserialize<'de> for PaymentType {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let c = char::deserialize(deserializer)?;
        let u = u8::try_from(c).map_err(de::Error::custom)?;
        PaymentType::try_from(u).map_err(de::Error::custom)
    }
}

impl Serialize for PaymentType {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        (*self as u8 as char).serialize(serializer)
    }
}

/// A reference data security type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SecurityType {
    /// Bond
    Bnd,
    /// Basket Warrant
    Bsw,
    /// Share Depository Certificate
    Cda,
    /// Chess Depository Interest
    Cdi,
    /// Convertible Notes
    Cn,
    /// Contingent Value Rights
    Cvr,
    /// Depository Receipts
    Dr,
    /// Distribution Rights
    Drt,
    /// Deferred Settlement Trading
    Dst,
    /// Equity Shares
    Eqs,
    /// Exchange Traded Commodities
    Etc,
    /// Exchange Traded Fund
    Etf,
    /// Letter of Allotment
    Loa,
    /// Mutual Fund
    Mf,
    /// Preferred Security
    Pfs,
    /// Preference Share
    Prf,
    /// Parallel Line
    Prl,
    /// Receipt
    Rcp,
    /// Redemption Rights
    Rdr,
    /// Redeemable Shares
    Rds,
    /// Structured Product
    Sp,
    /// Subscription Receipts
    Srt,
    /// Second Trading Line
    Stl,
    /// Stapled Security
    Stp,
    /// Tradeable Rights
    Trt,
    /// Tendered Shares Security
    Tss,
    /// Units
    Unt,
    /// Warrants
    War,
    /// When Distributed
    Wd,
    /// When Issued
    Wis,
}
impl SecurityType {
    /// Converts a SecurityType to its `str` code.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Bnd => "BND",
            Self::Bsw => "BSW",
            Self::Cda => "CDA",
            Self::Cdi => "CDI",
            Self::Cn => "CN",
            Self::Cvr => "CVR",
            Self::Dr => "DR",
            Self::Drt => "DRT",
            Self::Dst => "DST",
            Self::Eqs => "EQS",
            Self::Etc => "ETC",
            Self::Etf => "ETF",
            Self::Loa => "LOA",
            Self::Mf => "MF",
            Self::Pfs => "PFS",
            Self::Prf => "PRF",
            Self::Prl => "PRL",
            Self::Rcp => "RCP",
            Self::Rdr => "RDR",
            Self::Rds => "RDS",
            Self::Sp => "SP",
            Self::Srt => "SRT",
            Self::Stl => "STL",
            Self::Stp => "STP",
            Self::Trt => "TRT",
            Self::Tss => "TSS",
            Self::Unt => "UNT",
            Self::War => "WAR",
            Self::Wd => "WD",
            Self::Wis => "WIS",
        }
    }
}

impl AsRef<str> for SecurityType {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::str::FromStr for SecurityType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "BND" => Ok(Self::Bnd),
            "BSW" => Ok(Self::Bsw),
            "CDA" => Ok(Self::Cda),
            "CDI" => Ok(Self::Cdi),
            "CN" => Ok(Self::Cn),
            "CVR" => Ok(Self::Cvr),
            "DR" => Ok(Self::Dr),
            "DRT" => Ok(Self::Drt),
            "DST" => Ok(Self::Dst),
            "EQS" => Ok(Self::Eqs),
            "ETC" => Ok(Self::Etc),
            "ETF" => Ok(Self::Etf),
            "LOA" => Ok(Self::Loa),
            "MF" => Ok(Self::Mf),
            "PFS" => Ok(Self::Pfs),
            "PRF" => Ok(Self::Prf),
            "PRL" => Ok(Self::Prl),
            "RCP" => Ok(Self::Rcp),
            "RDR" => Ok(Self::Rdr),
            "RDS" => Ok(Self::Rds),
            "SP" => Ok(Self::Sp),
            "SRT" => Ok(Self::Srt),
            "STL" => Ok(Self::Stl),
            "STP" => Ok(Self::Stp),
            "TRT" => Ok(Self::Trt),
            "TSS" => Ok(Self::Tss),
            "UNT" => Ok(Self::Unt),
            "WAR" => Ok(Self::War),
            "WD" => Ok(Self::Wd),
            "WIS" => Ok(Self::Wis),
            _ => Err(Error::bad_arg(
                "s",
                format!("no SecurityType variant associated with {s}"),
            )),
        }
    }
}

impl<'de> Deserialize<'de> for SecurityType {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let str = String::deserialize(deserializer)?;
        FromStr::from_str(&str).map_err(de::Error::custom)
    }
}

impl Serialize for SecurityType {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_str().serialize(serializer)
    }
}

/// The type of voting.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Voting {
    /// Limited Voting
    Limited = b'L',
    /// Multiple Voting
    Multiple = b'M',
    /// No Voting
    No = b'N',
    /// Voting
    Voting = b'V',
}
impl From<Voting> for u8 {
    fn from(value: Voting) -> u8 {
        value as u8
    }
}

impl From<Voting> for char {
    fn from(value: Voting) -> char {
        u8::from(value) as char
    }
}

impl TryFrom<u8> for Voting {
    type Error = Error;

    fn try_from(value: u8) -> std::result::Result<Self, Error> {
        match value {
            b'L' => Ok(Self::Limited),
            b'M' => Ok(Self::Multiple),
            b'N' => Ok(Self::No),
            b'V' => Ok(Self::Voting),
            _ => Err(Error::bad_arg(
                "value",
                format!("no Voting variant associated with {value:?}"),
            )),
        }
    }
}

impl<'de> Deserialize<'de> for Voting {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let c = char::deserialize(deserializer)?;
        let u = u8::try_from(c).map_err(de::Error::custom)?;
        Voting::try_from(u).map_err(de::Error::custom)
    }
}

impl Serialize for Voting {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        (*self as u8 as char).serialize(serializer)
    }
}
