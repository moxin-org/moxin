use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;

    ICON_TICK = dep("crate://self/resources/images/tick.png")

    ModelAttributeTag = <RoundedView> {
        width: Fit,
        height: Fit,
        padding: {top: 6, bottom: 6, left: 10, right: 10}

        spacing: 5,
        draw_bg: {
            radius: 2.0,
        }

        caption = <Label> {
            draw_text: {
                text_style: <REGULAR_FONT>{font_size: 9},
                color: #1D2939
            }
        }
    }

    ModelInfo = <View> {
        padding: 16,
        spacing: 10,
        width: Fill,
        height: Fit,
        align: {x: 0.0, y: 0.5},

        cursor: Hand,

        label = <Label> {
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 11},
                color: #000
            }
        }

        architecture_tag = <ModelAttributeTag> {
            draw_bg: {
                color: #DDD7FF,
            }
        }

        params_size_tag = <ModelAttributeTag> {
            draw_bg: {
                color: #D1F4FC,
            }
        }

        file_size_tag = <ModelAttributeTag> {
            caption = {
                draw_text:{
                    color: #000
                }
            }
            draw_bg: {
                color: #fff,
                border_color: #B4B4B4,
                border_width: 1.0,
            }
        }

        icon_tick_tag = <RoundedView> {
            align: {x: 1.0, y: 0.5}, 
            visible: false,
            icon_tick = <Image> {
                width: 14,
                height: 14,
                source: (ICON_TICK),
            }
        }
    }
}

