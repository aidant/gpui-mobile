use gpui::{MonochromeSprite, PolychromeSprite, SubpixelSprite};

#[test]
fn sprite_shader_layouts_match_gpui() {
    let common = include_str!("../src/shaders.wgsl");
    let storage = include_str!("../src/shaders_storage.wgsl");
    let webgl = include_str!("../src/shaders_webgl.wgsl");
    let subpixel = include_str!("../src/shaders_subpixel.wgsl");
    for (source, capabilities) in [
        (
            format!("{common}\n{storage}"),
            naga::valid::Capabilities::empty(),
        ),
        (
            format!("{common}\n{webgl}"),
            naga::valid::Capabilities::empty(),
        ),
        (
            format!("enable dual_source_blending;\n{common}\n{storage}\n{subpixel}"),
            naga::valid::Capabilities::DUAL_SOURCE_BLENDING,
        ),
    ] {
        let module = naga::front::wgsl::parse_str(&source).expect("shader should parse");
        naga::valid::Validator::new(naga::valid::ValidationFlags::all(), capabilities)
            .validate(&module)
            .expect("shader should validate");

        // Sizes alone cannot detect reordered fields. GPUI uploads these structs
        // directly, so every shader member must match its Rust offset.
        macro_rules! assert_layout {
            ($ty:ident, $($field:ident),+ $(,)?) => {{
                let ty = module.types.iter()
                    .find(|(_, ty)| ty.name.as_deref() == Some(stringify!($ty)))
                    .expect("sprite type should exist").1;
                let naga::TypeInner::Struct { members, span } = &ty.inner else {
                    panic!("sprite type should be a struct");
                };
                assert_eq!(*span as usize, std::mem::size_of::<$ty>());
                $(
                    let member = members.iter()
                        .find(|member| member.name.as_deref() == Some(stringify!($field)))
                        .expect("sprite member should exist");
                    assert_eq!(member.offset as usize, std::mem::offset_of!($ty, $field),
                        "{}.{}", stringify!($ty), stringify!($field));
                )+
            }};
        }
        assert_layout!(
            MonochromeSprite,
            order,
            pad,
            tile,
            bounds,
            content_mask,
            color,
            transformation
        );
        assert_layout!(
            PolychromeSprite,
            order,
            pad,
            tile,
            grayscale,
            opacity,
            bounds,
            content_mask,
            corner_radii
        );
        if capabilities.contains(naga::valid::Capabilities::DUAL_SOURCE_BLENDING) {
            assert_layout!(
                SubpixelSprite,
                order,
                pad,
                tile,
                bounds,
                content_mask,
                color,
                transformation
            );
        }
    }
}
