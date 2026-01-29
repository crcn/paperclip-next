/// Tests for Bundle with MockFileSystem
use crate::*;
use paperclip_parser::parse_with_path;
use std::path::PathBuf;

#[cfg(test)]
mod filesystem_tests {
    use super::*;

    #[test]
    fn test_mock_filesystem_import_resolution() {
        // Test that imports are resolved correctly with mock file system
        let mut bundle = Bundle::new();
        let mut mock_fs = paperclip_bundle::MockFileSystem::new();

        // Add files to mock filesystem
        mock_fs.add_file(PathBuf::from("/project/colors.pc"));
        mock_fs.add_file(PathBuf::from("/project/main.pc"));

        // Create colors file
        let colors_source = r#"
            public token primaryColor #3366FF
        "#;
        let colors_doc = parse_with_path(colors_source, "/project/colors.pc").unwrap();
        bundle.add_document(PathBuf::from("/project/colors.pc"), colors_doc);

        // Create main file that imports colors
        let main_source = r#"
            import "./colors.pc" as colors
        "#;
        let main_doc = parse_with_path(main_source, "/project/main.pc").unwrap();
        bundle.add_document(PathBuf::from("/project/main.pc"), main_doc);

        // Build dependencies with mock file system
        let result = bundle.build_dependencies_with_fs(&PathBuf::from("/project"), &mock_fs);
        assert!(result.is_ok(), "Should build dependencies with mock FS");

        // Verify the alias mapping was created
        let deps = bundle.get_dependencies(&PathBuf::from("/project/main.pc"));
        assert!(deps.is_some());
        assert_eq!(deps.unwrap().len(), 1, "Should have one dependency");
    }

    #[test]
    fn test_alias_resolution_with_mock_fs() {
        // Test that namespaced lookups work correctly with alias mapping
        let mut bundle = Bundle::new();
        let mut mock_fs = paperclip_bundle::MockFileSystem::new();

        mock_fs.add_file(PathBuf::from("/app/theme.pc"));
        mock_fs.add_file(PathBuf::from("/app/button.pc"));

        // Create theme file
        let theme_source = r#"
            public style fontBase {
                font-family: Inter
            }
        "#;
        let theme_doc = parse_with_path(theme_source, "/app/theme.pc").unwrap();
        bundle.add_document(PathBuf::from("/app/theme.pc"), theme_doc);

        // Create button file that imports theme
        let button_source = r#"
            import "./theme.pc" as theme

            public style buttonStyle extends theme.fontBase {
                padding: 8px
            }
        "#;
        let button_doc = parse_with_path(button_source, "/app/button.pc").unwrap();
        bundle.add_document(PathBuf::from("/app/button.pc"), button_doc);

        // Build dependencies
        bundle
            .build_dependencies_with_fs(&PathBuf::from("/app"), &mock_fs)
            .unwrap();

        // Test that we can find the style using the alias
        let style = bundle.find_style("theme.fontBase", &PathBuf::from("/app/button.pc"));
        assert!(style.is_some(), "Should find theme.fontBase via alias");

        let (style_decl, file_path) = style.unwrap();
        assert_eq!(style_decl.name, "fontBase");
        assert_eq!(file_path, PathBuf::from("/app/theme.pc"));
    }

    #[test]
    fn test_multiple_imports_with_different_aliases() {
        // Test that multiple imports with different aliases work correctly
        let mut bundle = Bundle::new();
        let mut mock_fs = paperclip_bundle::MockFileSystem::new();

        mock_fs.add_file(PathBuf::from("/src/colors.pc"));
        mock_fs.add_file(PathBuf::from("/src/fonts.pc"));
        mock_fs.add_file(PathBuf::from("/src/app.pc"));

        // Colors file
        let colors_source = r#"
            public token primary #FF0000
        "#;
        let colors_doc = parse_with_path(colors_source, "/src/colors.pc").unwrap();
        bundle.add_document(PathBuf::from("/src/colors.pc"), colors_doc);

        // Fonts file
        let fonts_source = r#"
            public style base {
                font-family: Arial
            }
        "#;
        let fonts_doc = parse_with_path(fonts_source, "/src/fonts.pc").unwrap();
        bundle.add_document(PathBuf::from("/src/fonts.pc"), fonts_doc);

        // App file with two imports
        let app_source = r#"
            import "./colors.pc" as colors
            import "./fonts.pc" as fonts

            public component App {
                render div {
                    style extends fonts.base {
                        color: colors.primary
                    }
                    text "Hello"
                }
            }
        "#;
        let app_doc = parse_with_path(app_source, "/src/app.pc").unwrap();
        bundle.add_document(PathBuf::from("/src/app.pc"), app_doc);

        // Build dependencies
        bundle
            .build_dependencies_with_fs(&PathBuf::from("/src"), &mock_fs)
            .unwrap();

        // Test both aliases work
        let color_token = bundle.find_token("colors.primary", &PathBuf::from("/src/app.pc"));
        assert!(color_token.is_some(), "Should find colors.primary");
        assert_eq!(color_token.unwrap().0.name, "primary");

        let font_style = bundle.find_style("fonts.base", &PathBuf::from("/src/app.pc"));
        assert!(font_style.is_some(), "Should find fonts.base");
        assert_eq!(font_style.unwrap().0.name, "base");
    }

    #[test]
    fn test_import_not_found_with_mock_fs() {
        // Test that missing imports are properly detected with mock FS
        let mut bundle = Bundle::new();
        let mock_fs = paperclip_bundle::MockFileSystem::new();

        // Only add main file, not the imported file
        let main_source = r#"
            import "./missing.pc" as missing
        "#;
        let main_doc = parse_with_path(main_source, "/test/main.pc").unwrap();
        bundle.add_document(PathBuf::from("/test/main.pc"), main_doc);

        // Build dependencies should fail
        let result = bundle.build_dependencies_with_fs(&PathBuf::from("/test"), &mock_fs);
        assert!(result.is_err(), "Should fail when import is not found");

        match result {
            Err(BundleError::ImportNotFound { import_path, .. }) => {
                assert_eq!(import_path, "./missing.pc");
            }
            _ => panic!("Expected ImportNotFound error"),
        }
    }

    #[test]
    fn test_asset_tracking_in_bundle() {
        // Test that assets are properly tracked at bundle level
        let mut bundle = Bundle::new();

        // Add an asset
        let asset = AssetReference {
            path: "/images/logo.png".to_string(),
            asset_type: AssetType::Image,
            resolved_path: PathBuf::from("/project/images/logo.png"),
            source_file: PathBuf::from("/project/main.pc"),
        };

        bundle.add_asset(asset.clone());

        // Verify asset is in bundle
        assert_eq!(bundle.unique_asset_count(), 1);
        let assets: Vec<_> = bundle.unique_assets().collect();
        assert_eq!(assets[0].path, "/images/logo.png");
        assert_eq!(assets[0].source_file, PathBuf::from("/project/main.pc"));
        assert!(matches!(assets[0].asset_type, AssetType::Image));

        // Verify we can query which files use this asset
        let users = bundle.asset_users("/images/logo.png").unwrap();
        assert_eq!(users.len(), 1);
        assert!(users.contains(&PathBuf::from("/project/main.pc")));
    }

    #[test]
    fn test_assets_from_multiple_files() {
        // Test that assets from multiple files are all tracked
        let mut bundle = Bundle::new();

        // Asset from first file
        bundle.add_asset(AssetReference {
            path: "/images/hero.jpg".to_string(),
            asset_type: AssetType::Image,
            resolved_path: PathBuf::from("/project/images/hero.jpg"),
            source_file: PathBuf::from("/project/home.pc"),
        });

        // Asset from second file
        bundle.add_asset(AssetReference {
            path: "/fonts/inter.woff2".to_string(),
            asset_type: AssetType::Font,
            resolved_path: PathBuf::from("/project/fonts/inter.woff2"),
            source_file: PathBuf::from("/project/theme.pc"),
        });

        assert_eq!(bundle.unique_asset_count(), 2);

        // Verify we can identify which file each asset came from
        let hero_asset = bundle
            .unique_assets()
            .find(|a| a.path == "/images/hero.jpg")
            .unwrap();
        assert_eq!(hero_asset.source_file, PathBuf::from("/project/home.pc"));

        let font_asset = bundle
            .unique_assets()
            .find(|a| a.path == "/fonts/inter.woff2")
            .unwrap();
        assert_eq!(font_asset.source_file, PathBuf::from("/project/theme.pc"));

        // Verify we can query assets by source file
        let home_assets = bundle.assets_for_file(&PathBuf::from("/project/home.pc"));
        assert_eq!(home_assets.len(), 1);
        assert_eq!(home_assets[0].path, "/images/hero.jpg");

        let theme_assets = bundle.assets_for_file(&PathBuf::from("/project/theme.pc"));
        assert_eq!(theme_assets.len(), 1);
        assert_eq!(theme_assets[0].path, "/fonts/inter.woff2");
    }

    #[test]
    fn test_asset_deduplication() {
        // Test that the same asset used by multiple files is deduplicated
        let mut bundle = Bundle::new();

        // Add hero.jpg from home.pc
        bundle.add_asset(AssetReference {
            path: "/images/hero.jpg".to_string(),
            asset_type: AssetType::Image,
            resolved_path: PathBuf::from("/project/images/hero.jpg"),
            source_file: PathBuf::from("/project/home.pc"),
        });

        // Add hero.jpg from about.pc (same asset, different source)
        bundle.add_asset(AssetReference {
            path: "/images/hero.jpg".to_string(),
            asset_type: AssetType::Image,
            resolved_path: PathBuf::from("/project/images/hero.jpg"),
            source_file: PathBuf::from("/project/about.pc"),
        });

        // Add hero.jpg from contact.pc
        bundle.add_asset(AssetReference {
            path: "/images/hero.jpg".to_string(),
            asset_type: AssetType::Image,
            resolved_path: PathBuf::from("/project/images/hero.jpg"),
            source_file: PathBuf::from("/project/contact.pc"),
        });

        // Should only have 1 unique asset
        assert_eq!(bundle.unique_asset_count(), 1);

        // But it should track all 3 source files that use it
        let users = bundle.asset_users("/images/hero.jpg").unwrap();
        assert_eq!(users.len(), 3);
        assert!(users.contains(&PathBuf::from("/project/home.pc")));
        assert!(users.contains(&PathBuf::from("/project/about.pc")));
        assert!(users.contains(&PathBuf::from("/project/contact.pc")));

        // Each file should show hero.jpg in their assets
        assert_eq!(
            bundle
                .assets_for_file(&PathBuf::from("/project/home.pc"))
                .len(),
            1
        );
        assert_eq!(
            bundle
                .assets_for_file(&PathBuf::from("/project/about.pc"))
                .len(),
            1
        );
        assert_eq!(
            bundle
                .assets_for_file(&PathBuf::from("/project/contact.pc"))
                .len(),
            1
        );
    }
}
