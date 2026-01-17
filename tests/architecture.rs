//! Architecture Verification Suite
//! 
//! Enforces the "Anatomy" of the Sovereign Organism.
//! Ensures organs remain decoupled and traits are correctly implemented.

#[cfg(test)]
mod architecture_tests {
    use rust_agency::tools::Tool;
    use rust_agency::memory::Memory;
    use rust_agency::orchestrator::queue::TaskQueue;
    
    // 1. ANATOMY CHECK: All Tools must be Thread-Safe (Send + Sync)
    // This ensures the "Hands" and "Senses" can work in parallel background threads.
    #[test]
    fn test_tools_are_thread_safe() {
        fn assert_send_sync<T: Send + Sync>() {}
        
        assert_send_sync::<rust_agency::tools::VisionTool>();
        assert_send_sync::<rust_agency::tools::HandsTool>();
        assert_send_sync::<rust_agency::tools::WalletTool>();
        assert_send_sync::<rust_agency::tools::MutationTool>();
        assert_send_sync::<rust_agency::tools::CodebaseTool>();
        assert_send_sync::<rust_agency::tools::WatchdogTool>();
    }

    // 2. ANATOMY CHECK: Critical Organs must be Thread-Safe
    #[test]
    fn test_organs_are_thread_safe() {
        fn assert_send_sync<T: Send + Sync>() {}

        // Stomach
        assert_send_sync::<rust_agency::memory::LocalVectorMemory>();
        assert_send_sync::<rust_agency::memory::MemoryManager>();
        
        // Muscles
        assert_send_sync::<rust_agency::orchestrator::queue::SqliteTaskQueue>();
        
        // Nervous System
        assert_send_sync::<rust_agency::orchestrator::homeostasis::HomeostasisEngine>();
        assert_send_sync::<rust_agency::orchestrator::healing::HealingEngine>();
        
        // Economy
        assert_send_sync::<rust_agency::orchestrator::metabolism::EconomicMetabolism>();
        
        // Identity
        assert_send_sync::<rust_agency::orchestrator::sovereignty::SovereignIdentity>();
    }

    // 3. STRUCTURAL INTEGRITY: Public API Check
    // Ensures the "Brain" (Supervisor) can access all organs
    #[test]
    fn test_supervisor_access() {
        // Compile-time check: Fields must be public
        #[allow(dead_code)]
        fn check_access(s: &rust_agency::orchestrator::Supervisor) {
            let _ = &s.task_queue;
            let _ = &s.sensory;
            let _ = &s.vocal_cords;
            let _ = &s.metabolism;
            let _ = &s.identity;
        }
    }

    // 4. DEPENDENCY RULE: Memory should be self-contained
    // We can't easily check module imports at runtime without a parser crate,
    // but we can verify trait bounds.
    #[test]
    fn test_memory_abstraction() {
        // Memory must NOT depend on TaskQueue directly (it's a passive store)
        // This is a compile-time check enforced by the type system, but we explicitly
        // document it here.
        fn assert_memory_trait<T: Memory>() {}
        assert_memory_trait::<rust_agency::memory::VectorMemory>();
    }
}
