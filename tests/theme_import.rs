use ttysvg::svg::import;
use ttysvg::svg::theme::Theme;

const BASE16: &str = r##"
scheme: "Ocean Deep"
author: "someone"
base00: "2b303b"
base01: "343d46"
base02: "4f5b66"
base03: "65737e"
base04: "a7adba"
base05: "c0c5ce"
base06: "dfe1e8"
base07: "eff1f5"
base08: "bf616a"
base09: "d08770"
base0A: "ebcb8b"
base0B: "a3be8c"
base0C: "96b5b4"
base0D: "8fa1b3"
base0E: "b48ead"
base0F: "ab7967"
"##;

const BASE16_NEW: &str = r##"
system: "base16"
name: "Ocean Deep"
author: "someone"
variant: "dark"
palette:
  base00: "#2b303b"
  base01: "#343d46"
  base02: "#4f5b66"
  base03: "#65737e"
  base04: "#a7adba"
  base05: "#c0c5ce"
  base06: "#dfe1e8"
  base07: "#eff1f5"
  base08: "#bf616a"
  base09: "#d08770"
  base0A: "#ebcb8b"
  base0B: "#a3be8c"
  base0C: "#96b5b4"
  base0D: "#8fa1b3"
  base0E: "#b48ead"
  base0F: "#ab7967"
"##;

const WINDOWS_TERMINAL: &str = r##"
{
  "schemes": [
    {
      "name": "Campbell",
      "background": "#0C0C0C",
      "foreground": "#CCCCCC",
      "cursorColor": "#FFFFFF",
      "black": "#0C0C0C", "red": "#C50F1F", "green": "#13A10E", "yellow": "#C19C00",
      "blue": "#0037DA", "purple": "#881798", "cyan": "#3A96DD", "white": "#CCCCCC",
      "brightBlack": "#767676", "brightRed": "#E74856", "brightGreen": "#16C60C",
      "brightYellow": "#F9F1A5", "brightBlue": "#3B78FF", "brightPurple": "#B4009E",
      "brightCyan": "#61D6D6", "brightWhite": "#F2F2F2"
    },
    {
      "name": "One Half Light",
      "background": "#FAFAFA",
      "foreground": "#383A42",
      "black": "#383A42", "red": "#E45649", "green": "#50A14F", "yellow": "#C18401",
      "blue": "#0184BC", "purple": "#A626A4", "cyan": "#0997B3", "white": "#FAFAFA",
      "brightBlack": "#4F525D", "brightRed": "#DF6C75", "brightGreen": "#98C379",
      "brightYellow": "#E4C07A", "brightBlue": "#61AFEF", "brightPurple": "#C577DD",
      "brightCyan": "#56B5C1", "brightWhite": "#FFFFFF"
    }
  ]
}
"##;

#[test]
fn a_base16_scheme_becomes_a_usable_theme() {
    let theme = import::base16(BASE16).unwrap();
    theme.validate().unwrap();

    assert_eq!(theme.name, "ocean-deep");
    assert_eq!(theme.dark.bg, "#2b303b", "background is base00");
    assert_eq!(theme.dark.fg, "#c0c5ce", "text is base05");
    assert_eq!(theme.dark.ansi[1], "#bf616a", "red is base08");
    assert_eq!(theme.dark.ansi[2], "#a3be8c", "green is base0b");
    assert_eq!(theme.dark.ansi[4], "#8fa1b3", "blue is base0d");
    assert_eq!(theme.dark.ansi[8], "#65737e", "bright black is base03");
    assert_eq!(theme.dark.ansi[15], "#eff1f5", "bright white is base07");
    assert!(!theme.has_light());
}

#[test]
fn both_base16_file_layouts_read_the_same() {
    let old = import::base16(BASE16).unwrap();
    let new = import::base16(BASE16_NEW).unwrap();
    assert_eq!(old.name, new.name);
    assert_eq!(old.dark.ansi, new.dark.ansi);
    assert_eq!(old.dark.bg, new.dark.bg);
}

#[test]
fn a_base16_scheme_missing_a_slot_says_which_one() {
    let cut = BASE16.replace("base0D: \"8fa1b3\"\n", "");
    let err = import::base16(&cut).unwrap_err().to_string();
    assert!(err.contains("base0D"), "{err}");
}

#[test]
fn windows_terminal_settings_import_every_scheme() {
    let themes = import::windows_terminal(WINDOWS_TERMINAL).unwrap();
    assert_eq!(themes.len(), 2);

    let campbell = &themes[0];
    campbell.validate().unwrap();
    assert_eq!(campbell.name, "campbell");
    assert_eq!(campbell.dark.bg, "#0c0c0c", "colors are lowercased");
    assert_eq!(campbell.dark.cursor(), "#ffffff");
    assert_eq!(campbell.dark.ansi[5], "#881798", "purple lands on magenta");
    assert_eq!(campbell.dark.ansi[15], "#f2f2f2");

    assert!(!import::is_light(campbell));
    assert!(
        import::is_light(&themes[1]),
        "one half light is a light theme"
    );
}

#[test]
fn a_bare_windows_terminal_scheme_works_without_the_wrapper() {
    let one = r##"{"name":"Solo","background":"#101010","foreground":"#e0e0e0",
      "black":"#101010","red":"#ff0000","green":"#00ff00","yellow":"#ffff00",
      "blue":"#0000ff","purple":"#ff00ff","cyan":"#00ffff","white":"#e0e0e0",
      "brightBlack":"#808080","brightRed":"#ff8080","brightGreen":"#80ff80",
      "brightYellow":"#ffff80","brightBlue":"#8080ff","brightPurple":"#ff80ff",
      "brightCyan":"#80ffff","brightWhite":"#ffffff"}"##;
    let themes = import::windows_terminal(one).unwrap();
    assert_eq!(themes.len(), 1);
    assert_eq!(themes[0].name, "solo");
}

#[test]
fn a_scheme_missing_a_color_is_rejected_by_name() {
    let broken = WINDOWS_TERMINAL.replace("\"cyan\": \"#3A96DD\", ", "");
    let err = import::windows_terminal(&broken).unwrap_err().to_string();
    assert!(err.contains("cyan"), "{err}");
}

#[test]
fn an_imported_theme_survives_a_trip_through_toml() {
    let theme = import::base16(BASE16).unwrap();
    let text = toml::to_string_pretty(&theme).unwrap();
    let back: Theme = toml::from_str(&text).unwrap();

    back.validate().unwrap();
    assert_eq!(back.name, theme.name);
    assert_eq!(back.dark.ansi, theme.dark.ansi);
    assert_eq!(back.dark.cursor, theme.dark.cursor);
    assert_eq!(back.dark.buttons, theme.dark.buttons);
    assert!(back.light.is_none());
}

#[test]
fn names_become_something_you_can_type() {
    assert_eq!(import::slug("Ocean Deep"), "ocean-deep");
    assert_eq!(import::slug("One Half Light"), "one-half-light");
    assert_eq!(import::slug("  Tomorrow  Night!! "), "tomorrow-night");
    assert_eq!(import::slug("base16-ocean.dark"), "base16-ocean-dark");
}
