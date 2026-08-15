//! Paleta de colores para axysBrowser.
//!
//! El objetivo de este módulo es que el aspecto claro/oscuro no dependa del
//! tema GTK instalado en el sistema (que puede no definir bien los alias de
//! color, o no aplicar correctamente la variante clara/oscura al fondo de la
//! ventana ni a los popups). En su lugar, definimos explícitamente dos
//! paletas propias con `@define-color` — incluyendo el fondo de la ventana
//! y de los popovers/desplegables — y las usamos en todo el CSS
//! personalizado. La estructura, el layout y el tamaño de los widgets no
//! cambian entre temas: solo los colores.

/// Devuelve la hoja de estilos GTK CSS completa para el tema pedido.
pub fn stylesheet(dark: bool) -> String {
    let palette = if dark { DARK_PALETTE } else { LIGHT_PALETTE };

    // El texto "dim" (subtítulos, etiquetas secundarias) se atenúa con
    // opacidad sobre @axys_fg. En oscuro un 0.62 sobre blanco se sigue
    // leyendo bien; en claro ese mismo 0.62 sobre negro da un gris que
    // cuesta de leer, así que en claro usamos una opacidad mucho más alta
    // para que el texto secundario siga siendo prácticamente negro.
    let dim_alpha = if dark { "0.62" } else { "0.92" };

    format!(
        r#"
        {palette}

        window {{
            background-color: @axys_bg;
            color: @axys_fg;
        }}

        /* Some GTK style classes (title-1, dim-label, the theme's own
           symbolic-icon tinting...) can define their own `color` with
           higher specificity than the plain `window` selector above,
           which is what made text and icons stay pale/low-contrast on
           some pages instead of following our own palette. We set them
           explicitly here so every page — not just the ones using our
           custom `.address-top`/`.tab`/etc. classes — is guaranteed to
           follow @axys_fg in both themes. */
        label {{
            color: @axys_fg;
        }}

        .dim-label {{
            color: alpha(@axys_fg, {dim_alpha});
        }}

        .title-1, .title-2, .title-3, .title-4 {{
            color: @axys_fg;
        }}

        /* Symbolic icons (back/forward/reload/menu/extensions/downloads...)
           are tinted by the widget's `color` property. Without this, they
           fell back to the system theme's own (broken) tint. */
        button,
        button image,
        button.flat,
        button.flat image {{
            color: @axys_fg;
        }}

        /* Popovers (the ☰ menu, and the dropdown lists in Settings/Tools)
           are separate top-level surfaces in GTK4, not children of
           `window`, so they need their own explicit colors. GTK4 uses a
           couple of different CSS node names for these depending on the
           widget (`popover`, or a plain `window` with a `.popup` class for
           some list-backed popups like GtkDropDown) — we cover every
           variant here, plus a universal `*` selector inside them, so
           nothing is left inheriting the system theme's own (broken)
           dark/light colors regardless of which node type it turns out
           to be. */
        popover,
        popover > contents,
        popover.menu,
        popover.background,
        window.popup,
        window.popup > contents {{
            background-color: @axys_bg;
            color: @axys_fg;
        }}

        popover *,
        window.popup * {{
            color: @axys_fg;
        }}

        popover separator,
        window.popup separator {{
            background-color: alpha(@axys_fg, 0.10);
            color: alpha(@axys_fg, 0.10);
        }}

        popover modelbutton,
        popover button.model,
        popover row,
        window.popup row {{
            background-color: transparent;
        }}

        popover modelbutton:hover,
        popover button.model:hover,
        popover row:hover,
        window.popup row:hover {{
            background-color: alpha(@axys_fg, 0.08);
        }}

        popover row:selected,
        window.popup row:selected {{
            background-color: alpha(@axys_accent, 0.18);
            color: @axys_fg;
        }}

        dropdown {{
            color: @axys_fg;
            background-color: alpha(@axys_fg, 0.045);
            border: 1px solid alpha(@axys_fg, 0.10);
            border-radius: 9px;
        }}

        dropdown:hover {{
            background-color: alpha(@axys_fg, 0.07);
        }}

        dropdown arrow {{
            color: @axys_fg;
        }}

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

// Gris oscuro estándar de GNOME/Adwaita. Si tu sistema mostraba un tono
// distinto antes de este cambio, dime el hexadecimal exacto y lo ajusto.
const DARK_PALETTE: &str = r#"
    @define-color axys_bg #242424;
    @define-color axys_fg #e9ebec;
    @define-color axys_accent #4c8bf5;
"#;

// #e6e6e6 = rgb(230, 230, 230), como pediste (ni blanco puro ni el 220 más
// oscuro que mencionaste como alternativa). Si lo quieres más oscuro,
// dime el hex y lo cambio directamente aquí.
const LIGHT_PALETTE: &str = r#"
    @define-color axys_bg #e6e6e6;
    @define-color axys_fg #000000;
    @define-color axys_accent #2f6fe4;
"#;
