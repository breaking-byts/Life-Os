# Life OS

A privacy-first, AI-powered personal productivity desktop application that learns your patterns and helps you optimize your life across academics, physical wellness, skills, and mental health.

Built with **Tauri 2**, **React 19**, **TypeScript**, **TanStack Router/Query**, and **SQLite**, featuring a sophisticated on-device machine learning system that adapts to your unique rhythms and goals.

---

## Table of Contents

- [Overview](#overview)
- [Key Features](#key-features)
- [Architecture](#architecture)
- [Technology Stack](#technology-stack)
- [AI/ML System](#aiml-system)
- [Getting Started](#getting-started)
- [Project Structure](#project-structure)
- [Database Schema](#database-schema)
- [Development](#development)
- [Contributing](#contributing)
- [License](#license)

---

## Overview

Life OS is designed to be your personal operating system for life management. Unlike cloud-based productivity apps, Life OS runs entirely on your device with no data leaving your machine. The integrated AI agent learns from your behavior patterns, circadian rhythms, energy levels, and historical performance to provide personalized, explainable recommendations.

### Philosophy

1. **Privacy First**: All data and ML inference stays local on your device
2. **Adaptive Intelligence**: The system learns your patterns, not generic advice
3. **Multi-Domain Optimization**: Balances academics, fitness, skills, and wellness
4. **Explainable AI**: Every recommendation includes reasoning and feature importance
5. **Long-term Thinking**: Optimizes for sustainable habits, not just immediate productivity

---

## Key Features

### Academic Tracking

- **Courses**: Track courses with color coding, credit hours, and weekly hour targets
- **Assignments**: Manage assignments with due dates, priorities (low/medium/high/critical), and completion tracking
- **Exams**: Schedule exams with location, duration, grades, and weight tracking
- **Study Sessions**: Pomodoro-style focus sessions linked to specific courses with automatic time aggregation

### Physical Wellness

- **Workout Logging**: Log workouts with detailed exercise tracking (sets, reps, weights, RPE)
- **Exercise Database**: Integration with [wger API](https://wger.de/) + custom exercise support
- **Personal Records**: Automatic PR detection and historical tracking
- **Workout Templates**: Create reusable templates for quick logging
- **Activity Heatmap**: GitHub-style visualization of workout consistency

### Skills Development

- **Skill Tracking**: Define skills with categories and weekly hour targets
- **Practice Logs**: Log practice sessions with duration, notes, and progress tracking
- **Level Progression**: Track skill advancement through cumulative practice hours

### Mental Wellness

- **Daily Check-ins**: Track mood (1-5) and energy levels (1-5) daily
- **Streak Tracking**: Visualize consistency across all activities
- **Recovery Detection**: AI monitors fatigue patterns and suggests recovery

### Weekly Reviews

- **Structured Reflection**: Capture weekly wins, areas for improvement, and learnings
- **Goal Alignment**: Track progress toward long-term objectives
- **Historical Browse**: Review past reflections for pattern recognition

### Productivity Tools

- **Pomodoro Timer**: Customizable work/break durations with desktop notifications
- **Big Three Goals**: Set and track 3 priority goals per day
- **Session Persistence**: Timer state persists across app reloads

### AI-Powered Recommendations

- **Personalized Suggestions**: Context-aware action recommendations
- **Explainability**: Every recommendation shows contributing factors
- **Semantic Memory**: Recalls similar past experiences and their outcomes
- **Confidence Levels**: Clear indication of prediction certainty

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                          React Frontend                              │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐   │
│  │  Dashboard  │ │  Academic   │ │  Physical   │ │   Skills    │   │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘   │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │              TanStack Query (State Management)               │   │
│  └─────────────────────────────────────────────────────────────┘   │
└──────────────────────────────┬──────────────────────────────────────┘
                               │ Tauri IPC
┌──────────────────────────────▼──────────────────────────────────────┐
│                          Rust Backend                                │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    Command Handlers (80+)                    │   │
│  └─────────────────────────────────────────────────────────────┘   │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐   │
│  │   Models    │ │  Services   │ │    Agent    │ │     ML      │   │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘   │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    SQLite (via SQLx)                         │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

### Frontend (React/TypeScript)

- **TanStack Router**: File-based routing with type-safe navigation
- **TanStack Query**: Server state management with caching and optimistic updates
- **Radix UI / shadcn/ui**: Accessible, customizable component library
- **Tailwind CSS 4**: Utility-first styling with CSS variables

### Backend (Rust/Tauri)

- **Tauri 2**: Secure, lightweight desktop app framework
- **SQLx**: Compile-time verified SQL queries with async support
- **Tokio**: High-performance async runtime

### AI/ML Pipeline

- **ONNX Runtime**: Local neural network inference with CoreML acceleration
- **LanceDB**: Vector database for semantic memory
- **Bayesian Linear Bandit**: Principled exploration vs exploitation

---

## Technology Stack

### Frontend Dependencies

| Package         | Version | Purpose                               |
| --------------- | ------- | ------------------------------------- |
| React           | 19.2    | UI framework with concurrent features |
| TypeScript      | 5.7     | Static type checking                  |
| TanStack Router | 1.132   | Type-safe file-based routing          |
| TanStack Query  | 5.64    | Async state management                |
| Tailwind CSS    | 4.0     | Utility-first CSS                     |
| Radix UI        | 1.4     | Accessible component primitives       |
| Recharts        | 3.6     | Data visualization                    |
| date-fns        | 4.1     | Date manipulation                     |
| Lucide React    | 0.562   | Icon library                          |
| Vite            | 7.1     | Build tool with HMR                   |

### Backend Dependencies

| Crate   | Version | Purpose               |
| ------- | ------- | --------------------- |
| Tauri   | 2.9.5   | Desktop app framework |
| SQLx    | 0.8     | Async SQL toolkit     |
| Tokio   | 1.x     | Async runtime         |
| Serde   | 1.0     | Serialization         |
| Chrono  | 0.4     | Date/time handling    |
| reqwest | 0.12    | HTTP client           |

### ML/AI Dependencies

| Crate      | Version    | Purpose                           |
| ---------- | ---------- | --------------------------------- |
| ort        | 2.0.0-rc.9 | ONNX Runtime bindings             |
| tokenizers | 0.21       | HuggingFace tokenization          |
| lancedb    | 0.15       | Vector database                   |
| ndarray    | 0.17       | N-dimensional arrays              |
| nalgebra   | 0.33       | Linear algebra (Bayesian updates) |
| arrow      | 53         | Columnar data format              |
| rand_distr | 0.4        | Statistical distributions         |

---

## AI/ML System

Life OS features a sophisticated "Maximum Intelligence Productivity Agent" that learns your patterns and provides personalized recommendations.

### System Components

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Intelligence Agent                               │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────────────┐   │
│  │ Rich Context  │  │ Hybrid Bandit │  │   Semantic Memory     │   │
│  │  (50+ dims)   │  │ (UCB + Bayes) │  │      (LanceDB)        │   │
│  └───────────────┘  └───────────────┘  └───────────────────────┘   │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────────────┐   │
│  │ Multi-Scale   │  │   Pattern     │  │     Embedding         │   │
│  │   Rewards     │  │    Miner      │  │      Service          │   │
│  └───────────────┘  └───────────────┘  └───────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

### Rich Context Features (50 dimensions)

The agent captures a comprehensive 50-dimensional context vector at decision time:

| Category              | Features                                                                                                               | Description           |
| --------------------- | ---------------------------------------------------------------------------------------------------------------------- | --------------------- |
| **Temporal** (6)      | hour_of_day, day_of_week, week_of_year, is_weekend, is_morning, is_evening                                             | Time-based context    |
| **Physiological** (6) | energy_level, mood_level, energy_trajectory, mood_trajectory, fatigue_accumulated, recovery_need                       | Physical/mental state |
| **Learning** (6)      | skill_momentum, practice_diversity, focus_trend, study_streak, pomodoros_today, hours_since_break                      | Learning patterns     |
| **Goals** (6)         | big_three_completion, weekly_progress, assignment_urgency, exam_proximity, goal_alignment, streak_days                 | Goal tracking         |
| **Circadian** (4)     | peak_focus_prob, creative_window, analytical_window, energy_prediction                                                 | Biological rhythms    |
| **Historical** (6)    | similar_context_outcome, action_success_rate, hour_productivity, day_productivity, cumulative_success, recent_momentum | Past performance      |
| **Workload** (6)      | active_assignments, overdue_count, due_this_week, study_balance, workout_recency, hours_since_workout                  | Current load          |
| **Interactions** (10) | Cross-feature multiplicative interactions                                                                              | Feature combinations  |

### Hybrid Contextual Bandit

The recommendation engine uses a two-phase approach:

**Phase 1: Bayesian Linear Regression (Days 1-90)**

- Thompson Sampling for exploration
- Analytical posterior updates (no training required)
- Instant learning from each interaction
- UCB (Upper Confidence Bound) for action selection

```
UCB = E[reward|context] + β × uncertainty
```

**Phase 2: Neural Network Ensemble (Month 3+)**

- Planned upgrade path when sufficient data collected
- Epistemic uncertainty via ensemble disagreement
- Periodic offline training

### Multi-Scale Reward System

Balances optimization across multiple timescales:

| Timescale     | Weight | Signals                                             |
| ------------- | ------ | --------------------------------------------------- |
| **Immediate** | 20%    | User satisfaction, completion rate, feedback        |
| **Daily**     | 30%    | Big 3 progress, study balance, workout completion   |
| **Weekly**    | 30%    | Goal alignment, consistency, weekly review insights |
| **Monthly**   | 20%    | Long-term retention, wellbeing trends, skill growth |

### Semantic Memory

Uses LanceDB vector database for experience-based reasoning:

- **Event Storage**: All user activities stored with embeddings
- **Similarity Search**: Finds relevant past experiences
- **Outcome Tracking**: Links actions to results
- **Context Enrichment**: Augments decisions with historical success rates

### Embedding Service

Local embedding generation using ONNX Runtime:

- **Model**: Qwen3 embedding model (1024 dimensions)
- **Acceleration**: CoreML on Apple Silicon, DirectML on Windows
- **Pooling**: Mean pooling with L2 normalization
- **Privacy**: All inference runs locally

### Available Agent Actions

| Category         | Actions                                                           |
| ---------------- | ----------------------------------------------------------------- |
| **Productivity** | start_pomodoro, study_session, tackle_assignment, deep_work_block |
| **Physical**     | do_workout, take_walk, stretch_break                              |
| **Wellness**     | do_checkin, take_break, meditation                                |
| **Skills**       | practice_skill, learn_new_skill                                   |
| **Reflection**   | weekly_review, plan_tomorrow, review_goals                        |

### Explainability

Every recommendation includes:

- **Top Contributing Features**: Which context features drove the decision
- **Feature Direction**: Whether each feature pushed toward or against the action
- **Similar Experiences**: Past situations with outcomes
- **Confidence Level**: low / medium / high based on uncertainty
- **Alternative Actions**: Other good options with reasoning

---

## Getting Started

### Prerequisites

- **Rust** (1.77.2 or later): [Install Rust](https://rustup.rs/)
- **Node.js** (20 or later): [Install Node.js](https://nodejs.org/)
- **Bun** (recommended) or npm/pnpm: [Install Bun](https://bun.sh/)

### System Requirements

| Platform | Requirements                              |
| -------- | ----------------------------------------- |
| macOS    | 10.15+ (Catalina), Apple Silicon or Intel |
| Windows  | Windows 10 (1803+) with WebView2          |
| Linux    | WebKitGTK 4.1+                            |

### Installation

1. **Clone the repository**

```bash
git clone https://github.com/yourusername/life-os.git
cd life-os
```

2. **Install frontend dependencies**

```bash
bun install
# or: npm install
```

3. **Run in development mode**

```bash
bun tauri:dev
# or: npm run tauri:dev
```

This will:

- Start the Vite dev server on port 3000
- Compile the Rust backend
- Launch the desktop application with hot reload

4. **Build for production**

```bash
bun tauri:build
# or: npm run tauri:build
```

The built application will be in `src-tauri/target/release/bundle/`.

### First Run

On first launch:

1. The SQLite database is created at `{app_config_dir}/life-os.sqlite`
2. Database migrations run automatically
3. The embedding model downloads on first ML operation (~500MB)
4. LanceDB vector store initializes for semantic memory

---

## Project Structure

```
life-os/
├── public/                    # Static assets
│
├── src/                       # React frontend
│   ├── components/
│   │   ├── academic/          # Course, assignment, exam components
│   │   ├── dashboard/         # Dashboard widgets and agent UI
│   │   ├── layout/            # App layout and navigation
│   │   ├── physical/          # Workout and exercise components
│   │   ├── skills/            # Skill tracking components
│   │   ├── ui/                # Reusable UI components (shadcn)
│   │   └── weekly/            # Weekly review components
│   │
│   ├── hooks/                 # Custom React hooks
│   │   ├── useAssignments.ts  # Assignment CRUD operations
│   │   ├── useCheckIn.ts      # Daily check-in operations
│   │   ├── useCourses.ts      # Course management
│   │   ├── useExams.ts        # Exam tracking
│   │   ├── useExercises.ts    # Exercise database
│   │   ├── useIntelligence.ts # AI agent integration
│   │   ├── useSessions.ts     # Study/practice sessions
│   │   ├── useSkills.ts       # Skill management
│   │   ├── useStats.ts        # Analytics queries
│   │   ├── useTimer.ts        # Pomodoro timer state
│   │   └── useWorkouts.ts     # Workout logging
│   │
│   ├── lib/                   # Utility functions
│   │   ├── tauri.ts           # Tauri IPC wrappers
│   │   ├── utils.ts           # General utilities
│   │   └── workout-utils.ts   # Workout calculations
│   │
│   ├── routes/                # TanStack Router pages
│   │   ├── __root.tsx         # Root layout
│   │   ├── dashboard.tsx      # Main dashboard
│   │   ├── academic.tsx       # Academic tracking
│   │   ├── physical.tsx       # Workout tracking
│   │   ├── skills.tsx         # Skills page
│   │   ├── weekly.tsx         # Weekly reviews
│   │   └── settings.tsx       # App settings
│   │
│   ├── types/                 # TypeScript type definitions
│   │   └── index.ts           # Shared types
│   │
│   ├── router.tsx             # Router configuration
│   ├── routeTree.gen.ts       # Generated route tree
│   └── styles.css             # Global styles
│
├── src-tauri/                 # Rust backend
│   ├── src/
│   │   ├── agent/             # Intelligence agent
│   │   │   ├── intelligence.rs # Main agent orchestration
│   │   │   ├── insights.rs    # Insight generation
│   │   │   └── mod.rs
│   │   │
│   │   ├── commands/          # Tauri command handlers
│   │   │   ├── analytics.rs   # Statistics queries
│   │   │   ├── assignments.rs # Assignment CRUD
│   │   │   ├── checkins.rs    # Check-in operations
│   │   │   ├── courses.rs     # Course management
│   │   │   ├── exams.rs       # Exam tracking
│   │   │   ├── exercises.rs   # Exercise database
│   │   │   ├── intelligence.rs # Agent commands
│   │   │   ├── sessions.rs    # Session tracking
│   │   │   ├── skills.rs      # Skill management
│   │   │   ├── weekly_reviews.rs
│   │   │   └── workouts.rs    # Workout logging
│   │   │
│   │   ├── db/                # Database layer
│   │   │   ├── migrations/    # SQL migration files
│   │   │   ├── connection.rs  # Connection management
│   │   │   └── mod.rs
│   │   │
│   │   ├── ml/                # Machine learning
│   │   │   ├── bandit_v2.rs   # Hybrid contextual bandit
│   │   │   ├── embedding.rs   # ONNX embedding service
│   │   │   ├── feature_store.rs # Feature engineering
│   │   │   ├── models.rs      # Reward models
│   │   │   ├── pattern_miner.rs # Behavioral patterns
│   │   │   ├── rich_features.rs # 50-dim context
│   │   │   ├── semantic_memory.rs # LanceDB integration
│   │   │   └── user_profile.rs # User modeling
│   │   │
│   │   ├── models/            # Data models
│   │   │   ├── assignment.rs
│   │   │   ├── checkin.rs
│   │   │   ├── course.rs
│   │   │   ├── exam.rs
│   │   │   ├── exercise.rs
│   │   │   ├── session.rs
│   │   │   ├── skill.rs
│   │   │   ├── user.rs
│   │   │   └── workout.rs
│   │   │
│   │   ├── services/          # External services
│   │   │   └── wger.rs        # wger API client
│   │   │
│   │   ├── lib.rs             # Library entry point
│   │   └── main.rs            # Application entry point
│   │
│   ├── Cargo.toml             # Rust dependencies
│   └── tauri.conf.json        # Tauri configuration
│
├── package.json               # Node.js dependencies
├── vite.config.ts             # Vite configuration
├── tsconfig.json              # TypeScript configuration
└── README.md                  # This file
```

---

## Database Schema

Life OS uses SQLite with migration-based schema evolution. Key tables:

### Core Data Tables

```sql
-- User profile
users (id, name, email, created_at, updated_at)

-- Academic tracking
courses (id, name, color, credit_hours, weekly_target_hours, ...)
assignments (id, course_id, title, due_date, priority, is_completed, ...)
exams (id, course_id, title, exam_date, location, grade, weight, ...)
sessions (id, reference_type, reference_id, duration_minutes, ...)

-- Skill tracking
skills (id, name, category, current_level, weekly_target_hours, ...)
practice_logs (id, skill_id, duration_minutes, notes, ...)

-- Physical wellness
workouts (id, date, duration_minutes, notes, ...)
workout_exercises (id, workout_id, exercise_id, sets_json, ...)
exercises_cache (id, name, description, muscle_groups, is_custom, ...)
workout_templates (id, name, exercises_json, ...)

-- Mental wellness
check_ins (id, date, mood, energy, notes, ...)
weekly_reviews (id, week_start, wins, improvements, notes, ...)
```

### Agent/ML Tables

```sql
-- Semantic memory
agent_memory_events (id, timestamp, event_type, content, embedding, outcome_score, ...)

-- Bayesian bandit
agent_linear_bandit (id, action_id, theta_mean, precision_matrix, ...)

-- Rich context snapshots
agent_rich_context (id, timestamp, context_features, ...)

-- Multi-scale rewards
agent_reward_log (id, action, context_id, immediate, daily, weekly, monthly, ...)

-- Recommendations tracking
agent_recommendations (id, action_recommended, confidence, was_accepted, outcome_score, ...)

-- Big 3 daily goals
agent_big_three (id, date, priority, title, is_completed, ...)

-- Agent configuration
agent_state (key, value_json)
```

### Database Location

| Platform | Path                                                           |
| -------- | -------------------------------------------------------------- |
| macOS    | `~/Library/Application Support/com.life-os.app/life-os.sqlite` |
| Windows  | `%APPDATA%\com.life-os.app\life-os.sqlite`                     |
| Linux    | `~/.config/com.life-os.app/life-os.sqlite`                     |

---

## Development

### Available Scripts

```bash
# Start development server
bun dev                  # Vite dev server only (port 3000)
bun tauri:dev            # Full Tauri app with hot reload

# Build
bun build                # Build frontend only
bun tauri:build          # Build full application

# Code quality
bun lint                 # Run ESLint
bun format               # Run Prettier
bun check                # Format + lint fix

# Testing
bun test                 # Run Vitest tests
```

### Adding a New Feature

1. **Create the data model** in `src-tauri/src/models/`
2. **Add migration** in `src-tauri/src/db/migrations/`
3. **Create commands** in `src-tauri/src/commands/`
4. **Register commands** in `src-tauri/src/lib.rs`
5. **Add TypeScript types** in `src/types/index.ts`
6. **Create hook** in `src/hooks/`
7. **Build components** in `src/components/`

### Debugging

```bash
# Enable Rust logging
RUST_LOG=debug bun tauri:dev

# Open DevTools
# Press Cmd+Shift+I (macOS) or Ctrl+Shift+I (Windows/Linux) in the app
```

### Database Migrations

Migrations run automatically on app startup. To add a new migration:

1. Create `NNN_description.sql` in `src-tauri/src/db/migrations/`
2. The migration runs on next app launch

---

## Roadmap

### Near-term

- [ ] Mobile companion app (React Native)
- [ ] Calendar integration (Google Calendar, Apple Calendar)
- [ ] Spaced repetition for academic content
- [ ] Export/import data

### Medium-term

- [ ] Neural network phase for bandit (Phase 2)
- [ ] Voice notes for check-ins
- [ ] Collaborative features (study groups)
- [ ] Plugin system for custom integrations

### Long-term

- [ ] Cross-device sync (optional, encrypted)
- [ ] Natural language interface
- [ ] Predictive scheduling
- [ ] Health device integration (Apple Health, Fitbit)

---

## Contributing

Contributions are welcome! Please follow these steps:

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes
4. Run tests and linting: `bun check && bun test`
5. Commit your changes: `git commit -m 'Add amazing feature'`
6. Push to the branch: `git push origin feature/amazing-feature`
7. Open a Pull Request

### Development Guidelines

- Follow existing code style (Prettier + ESLint for TS, rustfmt for Rust)
- Write meaningful commit messages
- Add tests for new functionality
- Update documentation as needed

---

## Acknowledgments

- [Tauri](https://tauri.app/) - Secure, lightweight desktop apps
- [TanStack](https://tanstack.com/) - Type-safe routing and state management
- [shadcn/ui](https://ui.shadcn.com/) - Beautiful, accessible components
- [wger](https://wger.de/) - Exercise database API
- [LanceDB](https://lancedb.com/) - Vector database for ML

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## Support

If you encounter any issues or have questions:

1. Check existing [GitHub Issues](https://github.com/yourusername/life-os/issues)
2. Open a new issue with detailed reproduction steps
3. For feature requests, describe the use case and expected behavior

---

<div align="center">

**Built with privacy and intelligence at its core.**

</div>
