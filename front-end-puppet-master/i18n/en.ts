/**
 * Kroma App Translations
 *
 * English translations for Kroma app.
 */

export default defineI18nLocale(() => ({
  app: {
    // Navigation
    dashboard: 'Dashboard',
    projects: 'Projects',
    providers: 'Providers',
    quickTools: 'Quick Tools',
    settings: 'Settings',

    // Dashboard
    dashboard: {
      title: 'Dashboard',
      subtitle: 'Welcome back, {name}!',
      recentProjects: 'Recent Projects',
      stats: {
        projects: 'Projects',
        runs: 'Runs',
        assets: 'Assets'
      }
    },

    // Projects
    projects: {
      title: 'Projects',
      subtitle: 'Manage your comic/graphic-novel projects',
      create: 'Create Project',
      edit: 'Edit Project',
      delete: 'Delete Project',
      name: 'Project Name',
      namePlaceholder: 'Enter project name',
      description: 'Description',
      descriptionPlaceholder: 'Enter project description (optional)',
      recent: 'Recent Projects',
      empty: {
        title: 'No projects yet',
        description: 'Create your first project to start generating images with consistent style and characters.'
      }
    },

    // Providers
    providers: {
      title: 'Providers',
      subtitle: 'Configure AI provider accounts',
      add: 'Add Provider',
      edit: 'Edit Provider',
      empty: {
        title: 'No providers yet',
        description: 'Add your first AI provider to start generating images.'
      },
      comingSoon: 'Provider Management Coming Soon',
      comingSoonDescription: 'You will be able to configure OpenAI, Stability, and other AI providers here.'
    },

    // Quick Tools
    quickTools: {
      subtitle: 'Quick image utilities without project context',
      description: 'Background removal, upscaling, and color correction',
      comingSoon: 'Quick Tools Coming Soon',
      comingSoonDescription: 'You will be able to use background removal, upscaling, and color correction tools here.'
    },

    // Settings
    settings: {
      subtitle: 'Application settings',
      comingSoon: 'Settings Coming Soon',
      comingSoonDescription: 'Application settings will be available here.'
    },

    // Runs
    runs: {
      title: 'Runs',
      subtitle: 'Generation run history',
      create: 'Trigger Run',
      empty: {
        title: 'No runs yet',
        description: 'Trigger your first generation run to start creating images.'
      },
      status: {
        pending: 'Pending',
        running: 'Running',
        completed: 'Completed',
        failed: 'Failed',
        cancelled: 'Cancelled'
      }
    },

    // Assets
    assets: {
      title: 'Assets'
    },

    // Quick Actions
    quickActions: 'Quick Actions'
  },

  // Common
  common: {
    viewAll: 'View All',
    cancel: 'Cancel',
    save: 'Save',
    delete: 'Delete',
    edit: 'Edit',
    create: 'Create',
    loading: 'Loading...',
    error: 'Error',
    success: 'Success'
  },

  // Auth
  auth: {
    changePassword: 'Change Password',
    logout: 'Logout'
  },

  // Theme
  common: {
    lightMode: 'Light Mode',
    darkMode: 'Dark Mode'
  }
}))
