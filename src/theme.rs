//! Paleta de colores para los widgets personalizados de axysBrowser.
//!
//! El objetivo de este módulo es que el aspecto claro/oscuro no dependa del
//! tema GTK instalado en el sistema (que puede no definir bien los alias de
//! color, o no refrescarse al vuelo). En su lugar, definimos explícitamente
//! dos paletas propias con `@define-color` y las usamos en todo el CSS
//! personalizado. La estructura, el layout y el tamaño de los widgets no
//! cambian entre temas: solo los colores.

/// Devuelve la hoja de estilos GTK CSS completa para el tema pedido.
pub fn stylesheet(dark: bool) -> String {
    let palette = if dark { DARK_PALETTE } else { LIGHT_PALETTE };

    format!(
        r#"
        {palette}

        .address-top {{
            min-height: 18px;
            padding: 5px 12px;
            border-radius: 16px;
            border: 1px solid alpha(@axys_fg, 0.08);
            background: alpha(@axys_fg, 0.045);
            box-shadow: none;
            font-size: 0.90em;
            color: @axys_fg;
        }}

        .address-top:focus {{
            border-color: alpha(@axys_accent, 0.45);
            background: alpha(@axys_fg, 0.07);
        }}

        .search-main {{
            min-height: 34px;
            padding: 10px 16px;
            border-radius: 18px;
            border: 1px solid alpha(@axys_fg, 0.10);
            background: alpha(@axys_fg, 0.055);
            box-shadow: none;
            font-size: 0.92em;
            color: @axys_fg;
        }}

        .search-main:focus {{
            border-color: alpha(@axys_accent, 0.55);
            background: alpha(@axys_fg, 0.08);
        }}

        .tab {{
            min-height: 30px;
            padding: 2px 4px 2px 9px;
            border-radius: 9px;
            background: transparent;
        }}

        .tab:hover {{
            background: alpha(@axys_fg, 0.06);
        }}

        .tab.active {{
            background: alpha(@axys_fg, 0.10);
        }}

        .tab-select {{
            padding: 2px 5px;
            border-radius: 7px;
            background: transparent;
            box-shadow: none;
            color: @axys_fg;
        }}

        .tab-close {{
            opacity: 0;
            min-width: 20px;
            min-height: 20px;
            padding: 0;
            border-radius: 6px;
        }}

        .tab:hover .tab-close,
        .tab.active .tab-close {{
            opacity: 1;
        }}

        .newtab-logo {{
            font-size: 3.8em;
            font-weight: 600;
            padding: 4px 18px;
            border-radius: 12px;
            background: transparent;
            box-shadow: none;
            color: @axys_fg;
        }}

        .newtab-logo:hover {{
            background: alpha(@axys_fg, 0.06);
        }}

        .newtab-logo:active {{
            background: alpha(@axys_fg, 0.10);
        }}

        button.flat {{
            min-width: 30px;
            min-height: 30px;
            padding: 4px;
            border-radius: 9px;
            box-shadow: none;
        }}

        button.flat:hover {{
            background: alpha(@axys_fg, 0.07);
        }}

        button.flat:active {{
            background: alpha(@axys_fg, 0.12);
        }}

        .private-banner {{
            padding: 6px 12px;
            border-radius: 12px;
            background: alpha(@axys_accent, 0.10);
            color: @axys_fg;
        }}
        "#
    )
}

const DARK_PALETTE: &str = r#"
    @define-color axys_fg #e9ebec;
    @define-color axys_accent #4c8bf5;
"#;

const LIGHT_PALETTE: &str = r#"
    @define-color axys_fg #1b1c1d;
    @define-color axys_accent #2f6fe4;
"#;
