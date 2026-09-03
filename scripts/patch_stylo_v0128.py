import os
V='vendor/stylo/'
p=V+'style/servo/media_features.rs'; s=open(p,encoding='utf-8').read()
s=s.replace('use crate::queries::values::{Orientation, PrefersColorScheme};','use crate::queries::values::{Orientation, PrefersColorScheme};\nuse crate::values::specified::color::ForcedColors;')
add='''
/// https://drafts.csswg.org/mediaqueries-5/#prefers-reduced-motion
#[derive(Clone, Copy, Debug, FromPrimitive, Parse, PartialEq, ToCss)]
#[repr(u8)]
enum PrefersReducedMotion {
    NoPreference,
    Reduce,
}
fn eval_prefers_reduced_motion(_: &Context, query_value: Option<PrefersReducedMotion>) -> bool {
    matches!(query_value, Some(PrefersReducedMotion::NoPreference))
}
/// https://drafts.csswg.org/mediaqueries-5/#prefers-reduced-transparency
#[derive(Clone, Copy, Debug, FromPrimitive, Parse, PartialEq, ToCss)]
#[repr(u8)]
enum PrefersReducedTransparency {
    NoPreference,
    Reduce,
}
fn eval_prefers_reduced_transparency(_: &Context, query_value: Option<PrefersReducedTransparency>) -> bool {
    matches!(query_value, Some(PrefersReducedTransparency::NoPreference))
}
/// https://drafts.csswg.org/mediaqueries-5/#prefers-contrast
#[derive(Clone, Copy, Debug, FromPrimitive, Parse, PartialEq, ToCss)]
#[repr(u8)]
enum PrefersContrast {
    More,
    Less,
    Custom,
    NoPreference,
}
fn eval_prefers_contrast(_: &Context, query_value: Option<PrefersContrast>) -> bool {
    matches!(query_value, Some(PrefersContrast::NoPreference))
}
/// https://drafts.csswg.org/mediaqueries-5/#forced-colors
fn eval_forced_colors(_: &Context, query_value: Option<ForcedColors>) -> bool {
    matches!(query_value, Some(ForcedColors::None))
}
/// https://drafts.csswg.org/mediaqueries-5/#inverted
#[derive(Clone, Copy, Debug, FromPrimitive, Parse, PartialEq, ToCss)]
#[repr(u8)]
enum InvertedColors {
    None,
    Inverted,
}
fn eval_inverted_colors(_: &Context, query_value: Option<InvertedColors>) -> bool {
    matches!(query_value, Some(InvertedColors::None))
}
/// https://drafts.csswg.org/mediaqueries-5/#dynamic-range
#[derive(Clone, Copy, Debug, FromPrimitive, Parse, PartialEq, ToCss)]
#[repr(u8)]
enum DynamicRange {
    Standard,
    High,
}
fn eval_dynamic_range(_: &Context, query_value: Option<DynamicRange>) -> bool {
    matches!(query_value, Some(DynamicRange::Standard))
}
/// Legacy `-ms-high-contrast`: never active.
#[derive(Clone, Copy, Debug, FromPrimitive, Parse, PartialEq, ToCss)]
#[repr(u8)]
enum MsHighContrast {
    None,
    Active,
    BlackOnWhite,
    WhiteOnBlack,
}
fn eval_ms_high_contrast(_: &Context, query_value: Option<MsHighContrast>) -> bool {
    matches!(query_value, Some(MsHighContrast::None))
}
'''
s=s.replace('\nbitflags! {', add+'\nbitflags! {',1)
feats='''    feature!(
        atom!("prefers-reduced-motion"),
        AllowsRanges::No,
        keyword_evaluator!(eval_prefers_reduced_motion, PrefersReducedMotion),
        FeatureFlags::empty(),
    ),
    feature!(
        atom!("prefers-reduced-transparency"),
        AllowsRanges::No,
        keyword_evaluator!(eval_prefers_reduced_transparency, PrefersReducedTransparency),
        FeatureFlags::empty(),
    ),
    feature!(
        atom!("prefers-contrast"),
        AllowsRanges::No,
        keyword_evaluator!(eval_prefers_contrast, PrefersContrast),
        FeatureFlags::empty(),
    ),
    feature!(
        atom!("forced-colors"),
        AllowsRanges::No,
        keyword_evaluator!(eval_forced_colors, ForcedColors),
        FeatureFlags::empty(),
    ),
    feature!(
        atom!("inverted-colors"),
        AllowsRanges::No,
        keyword_evaluator!(eval_inverted_colors, InvertedColors),
        FeatureFlags::empty(),
    ),
    feature!(
        atom!("dynamic-range"),
        AllowsRanges::No,
        keyword_evaluator!(eval_dynamic_range, DynamicRange),
        FeatureFlags::empty(),
    ),
    feature!(
        atom!("-ms-high-contrast"),
        AllowsRanges::No,
        keyword_evaluator!(eval_ms_high_contrast, MsHighContrast),
        FeatureFlags::empty(),
    ),
];'''
assert s.count('\n];')==1
s=s.replace('\n];','\n'+feats,1)
s=s.replace('[QueryFeatureDescription; 15]','[QueryFeatureDescription; 22]')
open(p,'w',encoding='utf-8').write(s)
p=V+'style/properties/longhands.toml'; s=open(p,encoding='utf-8').read()
assert '[text-wrap-style]\nstruct = "inherited_text"\nengine = "gecko"' in s
s=s.replace('[text-wrap-style]\nstruct = "inherited_text"\nengine = "gecko"','[text-wrap-style]\nstruct = "inherited_text"\nservo_pref = "layout.unimplemented"')
open(p,'w',encoding='utf-8').write(s)
p=V+'style/properties/shorthands.toml'; s=open(p,encoding='utf-8').read()
assert '[text-wrap]\nengine = "gecko"\n' in s
s=s.replace('[text-wrap]\nengine = "gecko"\n','[text-wrap]\nservo_pref = "layout.unimplemented"\n')
open(p,'w',encoding='utf-8').write(s)
p='Cargo.toml'; s=open(p,encoding='utf-8').read()
assert '[patch.' not in s
s=s.replace('[dependencies]\n','[dependencies]\nstylo_static_prefs = { path = "vendor/stylo/stylo_static_prefs", optional = true }\n',1)
s=s.replace('servo-real = ["dep:servo",','servo-real = ["dep:servo", "dep:stylo_static_prefs",',1)
crates={'selectors':'selectors','servo_arc':'servo_arc','stylo':'style','stylo_atoms':'stylo_atoms','stylo_derive':'style_derive','stylo_dom':'stylo_dom','stylo_malloc_size_of':'malloc_size_of','stylo_static_prefs':'stylo_static_prefs','stylo_traits':'style_traits','to_shmem':'to_shmem','to_shmem_derive':'to_shmem_derive'}
s+='\n[patch."https://github.com/servo/stylo"]\n'+''.join('%s = { path = "vendor/stylo/%s" }\n'%(k,v) for k,v in crates.items())
open(p,'w',encoding='utf-8').write(s)
p='src/platform/servo_real.rs'; s=open(p,encoding='utf-8').read()
old='    p.layout_unimplemented = true;\n'
new='''    p.layout_unimplemented = true;
    stylo_static_prefs::set_pref!("layout.css.has-selector.enabled", true);
    stylo_static_prefs::set_pref!("layout.css.nth-child-of.enabled", true);
    stylo_static_prefs::set_pref!("layout.css.starting-style-at-rules.enabled", true);
    stylo_static_prefs::set_pref!("layout.css.light-dark.images.enabled", true);
'''
assert old in s; s=s.replace(old,new,1)
open(p,'w',encoding='utf-8',newline='').write(s)
print('patched')
