# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

```bash
# Install dependencies (requires pnpm)
pnpm install

# Start development server
pnpm run dev

# Build for production
pnpm build

# Preview production build locally
pnpm preview

# Linting
pnpm run lint:eslint      # ESLint for Vue/TS/TSX files
pnpm run lint:prettier    # Prettier formatting
pnpm run lint:stylelint   # Stylelint for styles

# Clean cache
pnpm clean:cache          # Clean Vite cache
pnpm clean:lib            # Remove node_modules
```

## Project Architecture

This is a Vue3 + TypeScript admin template using Naive UI, Alova for HTTP requests, and Pinia for state management.

### Key Technologies
- **Vue 3.5+** with Composition API
- **TypeScript 4.9+** for type safety
- **Naive UI 2.43+** as the component library
- **Alova 3.3+** for API requests (modern HTTP client with caching)
- **Pinia** for state management
- **Vite** for build tooling

### Directory Structure

```
src/
├── api/              # API service layer organized by module
├── components/       # Reusable components (Form, Table, Modal, Upload, etc.)
├── directives/       # Custom Vue directives (e.g., v-permission)
├── hooks/            # Composable functions
├── layout/           # Layout components (Logo, AsideMenu, PageHeader, TabsView)
├── router/           # Vue Router config with dynamic route modules
│   └── modules/      # Route definitions split by feature
├── store/            # Pinia stores
│   └── modules/      # user, asyncRoute, designSetting, tabsView, etc.
├── styles/           # Global styles and theme configurations
├── utils/            # Utilities including Alova HTTP client setup
└── views/            # Page components
```

### Authentication & Permission System

**Two Permission Modes** (configured in `src/store/modules/projectSetting.ts`):
1. **FIXED**: Frontend static route filtering - routes defined in `src/router/modules/` are filtered by user permissions
2. **BACK**: Dynamic menu loading from backend via `/api/menus` endpoint

**Permission Directive**: `v-permission` in `src/directives/permission.ts` can hide/disable elements based on user permissions.

**Route Guards** (`src/router/guards.ts`):
- Validates token before navigation
- Dynamically adds routes based on permissions
- Manages keep-alive caching for components

### State Management (Pinia)

Key stores in `src/store/modules/`:
- **user.ts**: Authentication token, permissions, user profile
- **asyncRoute.ts**: Dynamic route generation and menu management
- **designSetting.ts**: Theme colors, dark mode, layout preferences
- **tabsView.ts**: Multi-tab navigation state
- **screenLock.ts**: Screen lock functionality

### Theme System

- **16+ built-in theme colors** configured in `src/settings/designSetting.ts`
- **Dark/light mode** support with Naive UI theme integration
- **Theme switching** updates CSS custom properties dynamically
- **Persistent settings** stored in localStorage

### API Layer with Alova

Alova HTTP client configuration in `src/utils/http/alova/index.ts`:
- Automatic token injection via interceptors
- Built-in mock support with `@alova/mock` adapter
- Centralized error handling with user-friendly messages
- Response transformation and caching

API modules in `src/api/` follow modular structure by feature.

### Routing Architecture

**Dynamic Route Loading**:
1. Routes defined in `src/router/modules/*.ts`
2. Loaded via `import.meta.glob` in router index
3. Filtered by user permissions (FIXED mode) OR loaded from backend (BACK mode)
4. Sorted by `meta.sort` property
5. Registered dynamically with `router.addRoute()`

**Route Meta Properties**:
- `title`: Display name
- `icon`: Menu icon
- `permissions`: Required permission array
- `keepAlive`: Whether to cache the component
- `sort`: Menu ordering
- `hideInMenu`: Exclude from menu rendering

### Component Architecture

**Reusable Components** in `src/components/`:
- **Form/**: Advanced form with validation
- **Table/**: Data table with editing, pagination, and actions
- **Modal/**: Modal dialog with hooks-based control
- **Upload/**: File upload with progress tracking

Components use TypeScript interfaces for props and emit events consistently.

### Mock Data

Mock system in `mock/` directory:
- Integrates with Alova Mock Adapter
- Define API responses in TypeScript
- Toggle between mock and real APIs easily
- Useful for development and prototyping

## Important Conventions

### Commit Message Format

Follows conventional commits (enforced by commitlint):
- `feat`: New features
- `fix`: Bug fixes
- `style`: Code style changes
- `perf`: Performance improvements
- `refactor`: Code refactoring
- `docs`: Documentation
- `test`: Tests
- `chore`: Dependencies, build config

Example: `feat(user): add OAuth login support`

### File Naming

- Vue components: PascalCase (e.g., `UserProfile.vue`)
- Utils/services: camelCase (e.g., `formatDate.ts`)
- Stores: camelCase (e.g., `userStore.ts`)

### TypeScript Usage

- Strong typing enforced - avoid `any`
- Define interfaces in component files or `types/` directory
- Use Vue 3 `PropType` for complex prop types

## Build Configuration

**vite.config.ts** highlights:
- Path aliases: `@/` → `src/`, `/#/` → `types/`
- Manual chunk splitting for optimization
- Proxy setup for API calls in development
- Environment variables via `dotenv`

## Browser Support

Modern browsers only (no IE). Local development recommends Chrome 80+.
