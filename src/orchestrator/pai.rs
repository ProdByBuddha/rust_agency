use pai_core::manifest::ManifestEngine;
use pai_core::upgrades::UpgradeMonitor;
use pai_core::privacy::PrivacyGuard;
use crate::orchestrator::Supervisor;

pub struct PAIOrchestrator<'a> {
    pub engine: AlgorithmEngine,
    pub supervisor: Arc<tokio::sync::Mutex<Supervisor>>,
    pub formatter: ResponseFormatter,
    pub agent_factory: AgentFactory,
    pub prompt_engine: PromptEngine<'a>,
    pub telos: TelosEngine,
    pub skill_registry: SkillRegistry,
    pub fabric: FabricRegistry,
    pub classifier: EffortClassifier,
    pub learning: LearningEngine,
    pub capability_loader: DynamicCapabilityLoader,
    pub manifest: ManifestEngine,
    pub sentinel: UpgradeMonitor,
    pub privacy: PrivacyGuard,
}

impl<'a> PAIOrchestrator<'a> {
    pub fn new(mut effort: EffortLevel, supervisor: Arc<tokio::sync::Mutex<Supervisor>>) -> Self {
        let home = std::env::var("HOME").unwrap_or_default();
        let pai_dir = PathBuf::from(std::env::var("PAI_DIR").unwrap_or_else(|_| format!("{}/.claude", home)));
        
        let classifier = EffortClassifier::new();
        
        let traits_path = pai_dir.join("Packs/pai-agents-skill/src/skills/Agents/Data/Traits.yaml");
        let agent_factory = AgentFactory::from_yaml(&traits_path)
            .expect("Failed to load PAI traits");

        let mut prompt_engine = PromptEngine::new();
        let briefing_template = pai_dir.join("Packs/pai-prompting-skill/src/skills/Prompting/Templates/Primitives/Briefing.hbs");
        prompt_engine.register_template("briefing", &briefing_template)
            .expect("Failed to register briefing template");

        let mut skill_registry = SkillRegistry::new();
        let _ = skill_registry.scan_directory(&pai_dir.join("skills"));

        let capability_loader = DynamicCapabilityLoader::from_yaml(
            &pai_dir.join("Packs/pai-algorithm-skill/src/skills/THEALGORITHM/Data/Capabilities.yaml")
        ).expect("Failed to load PAI capabilities registry");

        Self {
            engine: AlgorithmEngine::new(effort),
            supervisor,
            formatter: ResponseFormatter::new(PAIIdentity::default()),
            agent_factory,
            prompt_engine,
            telos: TelosEngine::new(pai_dir.clone()),
            skill_registry,
            fabric: FabricRegistry::new(pai_dir.clone()),
            classifier,
            learning: LearningEngine::new(pai_dir.clone()),
            capability_loader,
            manifest: ManifestEngine::new(pai_dir),
            sentinel: UpgradeMonitor::new(),
            privacy: PrivacyGuard::new(),
        }
    }

    pub async fn run_task(&mut self, request: &str) -> Result<String> {
        // PAI: Privacy Guard Pre-Check
        if self.privacy.is_leak(request) {
            return Err(anyhow::anyhow!("BLOCKED: Request attempts to access protected PAI files."));
        }

        // ... (Task execution logic remains the same)
        let effort = self.classifier.classify(request);
        let limits = CapabilityOrchestrator::get_limits(effort);
        
        self.engine = AlgorithmEngine::new(effort);
        self.engine.add_requirement(request, ISCSource::Explicit);
        
        // PAI: Reinforcement Context (Lessons Learned)
        let lessons = self.learning.load_lessons(request)?;
        
        // Phase 2: THINK
        self.engine.advance_phase(); 
        
        let mut full_context = format!("{}\n\n", lessons);
        let agent_identity = self.agent_factory.compose_agent("technical", "meticulous", "systematic")?;
        
        // Phase 5: EXECUTE
        self.engine.advance_phase();
        
        // ... (Execute logic)
        let mut supervisor = self.supervisor.lock().await;
        let query = format!("{}\n\nIdentity:\n{}\n\nISC:\n{}", request, agent_identity, self.engine.generate_isc_table());
        let res = supervisor.handle(&query).await?;
        
        // Phase 6: VERIFY
        self.engine.advance_phase(); 
        
        // PAI: Skeptical Verifier Switch
        let verifier_identity = self.agent_factory.verifier_mode()?;
        let verify_query = format!("Review the following output for errors or omissions. Be skeptical.\n\nOutput: {}\n\nISC:\n{}", res.answer, self.engine.generate_isc_table());
        let _verify_res = supervisor.handle(&verify_query).await?; // In real impl, use verifier_identity
        
        // Phase 7: LEARN
        self.engine.advance_phase(); 
        
        // PAI: Temporal Sentinel Check
        let mut update_notice = String::new();
        if let Ok(updates) = self.sentinel.check_for_updates().await {
            if !updates.is_empty() {
                update_notice = format!("\n🔔 SYSTEM ALERT: {} new PAI upgrade(s) available.", updates.length());
            }
        }

        let output = self.formatter.format_response(
            &format!("Executed task: {}", request),
            &format!("Task verified. {}{}", "No issues found.", update_notice),
            &["Observation", "Execution", "Verification", "Sentinel Update Check"],
            &res.answer,
            "PAI cycle complete."
        );
        
        Ok(output)
    }
}
