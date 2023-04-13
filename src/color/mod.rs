use hex::*;

/**
 * An RGBA color
 */
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    /**
     * Converts a 3-or-4 byte rgb(a) hex string into a Color
     */
    pub fn from_hex_str(hex_str: &str) -> Color {
        if hex_str.len() < 7 {
            panic!("Hex color string too short.");
        }

        if hex_str.chars().nth(0).unwrap() != '#' {
            panic!("Hex color must start with #")
        }

        let decoded_values = hex::decode(&hex_str[1..]).expect("Invalid hex string");
        let num_values = decoded_values.len();
        if num_values < 3 || num_values > 4 {
            panic!("Must be 3 or 4 hex bytes.");
        }

        Color {
            r: decoded_values[0],
            g: decoded_values[1],
            b: decoded_values[2],
            a: if num_values == 3 { 0xff } else { decoded_values[3] },
        }
    }

    /**
     * Returns a copy of this color with brightness multiplied by the given fraction
     */
    pub fn brightness(&self, adjustment: f64) -> Color {
        Color {
            r: (self.r as f64 * adjustment) as u8,
            g: (self.g as f64 * adjustment) as u8,
            b: (self.b as f64 * adjustment) as u8,
            a: self.a,
        }
    }
}

impl ToString for Color {
    fn to_string(&self) -> String {
        format!("#{}", hex::encode(vec![
            self.r,
            self.g,
            self.b,
            self.a,
        ]))
    }
}

impl From<&str> for Color {
    fn from(value: &str) -> Self {
        Color::from_hex_str(value)
    }
}

impl From<String> for Color {
    fn from(value: String) -> Self {
        Color::from_hex_str(value.as_str())
    }
}
