//! Supported ECB currencies and Alfred presentation metadata.

/// A supported ECB currency.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Currency {
    code: &'static str,
    name: &'static str,
    flag: &'static str,
}

impl Currency {
    const fn new(code: &'static str, name: &'static str, flag: &'static str) -> Self {
        Self { code, name, flag }
    }

    /// Returns the ISO 4217 code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        self.code
    }

    /// Returns the historical display name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        self.name
    }

    /// Returns the flag emoji used by the workflow.
    #[must_use]
    pub const fn flag(self) -> &'static str {
        self.flag
    }

    /// Finds a currency using a case-insensitive ISO code.
    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        CURRENCIES
            .iter()
            .copied()
            .find(|currency| currency.code.eq_ignore_ascii_case(code))
    }
}

/// Currencies retained in the Dart workflow's stable display order.
pub const CURRENCIES: [Currency; 31] = [
    Currency::new("AUD", "Australian dollar", "🇦🇺"),
    Currency::new("BRL", "Brazilian real", "🇧🇷"),
    Currency::new("CAD", "Canadian dollar", "🇨🇦"),
    Currency::new("CHF", "Swiss franc", "🇨🇭"),
    Currency::new("CNY", "Chinese yuan renminbi", "🇨🇳"),
    Currency::new("CZK", "Czech koruna", "🇨🇿"),
    Currency::new("DKK", "Danish krone", "🇩🇰"),
    Currency::new("EUR", "Euro", "🇪🇺"),
    Currency::new("GBP", "Pound sterling", "🇬🇧"),
    Currency::new("HKD", "Hong Kong dollar", "🇭🇰"),
    Currency::new("HUF", "Hungarian forint", "🇭🇺"),
    Currency::new("IDR", "Indonesian rupiah", "🇮🇩"),
    Currency::new("ILS", "Israeli shekel", "🇮🇱"),
    Currency::new("INR", "Indian rupee", "🇮🇳"),
    Currency::new("ISK", "Icelandic krona", "🇮🇸"),
    Currency::new("JPY", "Japanese yen", "🇯🇵"),
    Currency::new("KRW", "South Korean won", "🇰🇷"),
    Currency::new("MXN", "Mexican peso", "🇲🇽"),
    Currency::new("MYR", "Malaysian ringgit", "🇲🇾"),
    Currency::new("NOK", "Norwegian krone", "🇳🇴"),
    Currency::new("NZD", "New Zealand dollar", "🇳🇿"),
    Currency::new("PHP", "Philippine peso", "🇵🇭"),
    Currency::new("PLN", "Polish zloty", "🇵🇱"),
    Currency::new("RON", "Romanian leu", "🇷🇴"),
    Currency::new("RUB", "Russian rouble", "🇷🇺"),
    Currency::new("SEK", "Swedish krona", "🇸🇪"),
    Currency::new("SGD", "Singapore dollar", "🇸🇬"),
    Currency::new("THB", "Thai baht", "🇹🇭"),
    Currency::new("TRY", "Turkish lira", "🇹🇷"),
    Currency::new("USD", "US dollar", "🇺🇸"),
    Currency::new("ZAR", "South African rand", "🇿🇦"),
];
