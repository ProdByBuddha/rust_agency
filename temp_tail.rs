    let mission_path = engine.get_file_path(pai_core::telos::TelosCategory::Mission);
    assert!(mission_path.to_str().unwrap().contains("MISSION.md"));
    
    let goals_path = engine.get_file_path(pai_core::telos::TelosCategory::Goals);
    assert!(goals_path.to_str().unwrap().contains("GOALS.md"));
}

#[test]
fn test_config_missing_customization_resilience() {
    let tmp = tempdir().unwrap();
    let base_path = tmp.path().join("base.json");
    let non_existent_custom = tmp.path().join("ghost.json");
    
    fs::write(&base_path, "{\"status\": \"ok\"}").unwrap();

    // Should load base normally without error if custom is missing
    let loaded = pai_core::config::ConfigLoader::load_with_customization(&base_path, &non_existent_custom).unwrap();
    assert_eq!(loaded["status"], "ok");
}

fn poll_hook(hook: &impl pai_core::PAIHook, event: &HookEvent) -> HookAction {
    tokio::runtime::Runtime::new().unwrap().block_on(hook.on_event(event)).unwrap()
}
