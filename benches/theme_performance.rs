use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lumos_desktop::ui::theming::engine::{
    Theme, ThemeEngine, color, ColorPalette, ThemeMode,
    dynamic_theme::{DynamicThemeManager, DynamicThemeSettings, DynamicColorShift},
    theme_effects::{EffectManager, EffectSettings, EffectType}
};
use std::sync::Arc;
use std::path::PathBuf;

fn create_test_theme(name: &str, mode: ThemeMode) -> Theme {
    let mut theme = Theme::default();
    theme.name = name.to_string();
    theme.mode = mode;
    theme.colors.primary = "#3f51b5".to_string();
    theme.colors.secondary = "#7986cb".to_string();
    theme.colors.accent = "#ff4081".to_string();
    theme.wallpaper = Some(PathBuf::from("/tmp/test_wallpaper.jpg"));
    theme
}

fn benchmark_theme_loading(c: &mut Criterion) {
    let mut group = c.benchmark_group("theme_loading");
    
    group.bench_function("create_default_theme", |b| {
        b.iter(|| black_box(Theme::default()))
    });
    
    let engine = ThemeEngine::new();
    let theme = create_test_theme("BenchTheme", ThemeMode::Light);
    
    group.bench_function("install_theme", |b| {
        b.iter(|| {
            let theme_clone = black_box(theme.clone());
            black_box(engine.install_theme(theme_clone));
        })
    });
    
    engine.install_theme(theme);
    
    group.bench_function("get_theme", |b| {
        b.iter(|| black_box(engine.get_current_theme()))
    });
    
    group.finish();
}

fn benchmark_color_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("color_operations");
    
    group.bench_function("rgb_from_hex", |b| {
        b.iter(|| black_box(color::RGB::from_hex("#3f51b5").unwrap()))
    });
    
    let rgb = color::RGB::new(63, 81, 181);
    
    group.bench_function("rgb_to_hsl", |b| {
        b.iter(|| black_box(rgb.to_hsl()))
    });
    
    let hsl = rgb.to_hsl();
    
    group.bench_function("hsl_to_rgb", |b| {
        b.iter(|| black_box(hsl.to_rgb()))
    });
    
    group.finish();
}

fn benchmark_dynamic_theme(c: &mut Criterion) {
    let mut group = c.benchmark_group("dynamic_theme");
    
    let engine = Arc::new(ThemeEngine::new());
    let mut dynamic_manager = DynamicThemeManager::new(engine.clone());
    
    // デフォルト設定で初期化
    let settings = DynamicThemeSettings::default();
    dynamic_manager.set_settings(settings);
    
    group.bench_function("apply_color_shift", |b| {
        let original = "#3f51b5";
        let shift = DynamicColorShift {
            hue_shift: 30.0,
            saturation_shift: 0.1,
            lightness_shift: 0.05,
            ..DynamicColorShift::default()
        };
        
        b.iter(|| {
            let color_str = black_box(original);
            let shift = black_box(&shift);
            
            if let Ok(rgb) = color::RGB::from_hex(color_str) {
                let mut hsl = rgb.to_hsl();
                
                // 色相シフト
                hsl.h = (hsl.h + shift.hue_shift) % 360.0;
                if hsl.h < 0.0 {
                    hsl.h += 360.0;
                }
                
                // 彩度シフト
                hsl.s = (hsl.s + shift.saturation_shift).max(0.0).min(1.0);
                
                // 明度シフト
                hsl.l = (hsl.l + shift.lightness_shift).max(0.0).min(1.0);
                
                black_box(hsl.to_rgb().to_hex())
            } else {
                black_box(color_str.to_string())
            }
        })
    });
    
    group.bench_function("update_environment", |b| {
        b.iter(|| black_box(dynamic_manager.update_environment()))
    });
    
    group.finish();
}

fn benchmark_theme_effects(c: &mut Criterion) {
    let mut group = c.benchmark_group("theme_effects");
    
    let mut effect_manager = EffectManager::new();
    
    group.bench_function("create_effect", |b| {
        let settings = EffectSettings::default();
        
        b.iter(|| {
            black_box(effect_manager.apply_effect(
                "bench_target",
                EffectType::FadeIn,
                Some(settings.clone())
            ));
        })
    });
    
    let mut settings = EffectSettings::default();
    settings.duration_ms = 1000;
    effect_manager.apply_effect("update_target", EffectType::FadeIn, Some(settings));
    
    group.bench_function("update_effects", |b| {
        b.iter(|| black_box(effect_manager.update_all()))
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_theme_loading,
    benchmark_color_operations,
    benchmark_dynamic_theme,
    benchmark_theme_effects
);
criterion_main!(benches); 