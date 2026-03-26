/**
 * OpenAPI 3.0 Specification
 *
 * Complete API documentation for Puppet Master.
 * Served at /api/docs/openapi.json
 */

export const openApiSpec = {
  openapi: '3.0.3',
  info: {
    title: 'Puppet Master API',
    description: `
Config-driven studio toolkit API for building client websites.

## Authentication
Most admin endpoints require cookie-based session authentication.
- Login via \`POST /api/auth/login\` to obtain a session
- Include CSRF token in \`x-csrf-token\` header for state-changing requests

## Rate Limiting
- Login: 5 attempts per 15 minutes per IP
- Contact form: 5 submissions per hour per IP

## Roles
- **master**: Full access (developer/agency)
- **admin**: Manage content + users (except master)
- **editor**: Content editing only
    `,
    version: '1.0.0',
    contact: {
      name: 'API Support'
    }
  },
  servers: [
    {
      url: '/api',
      description: 'API Server'
    }
  ],
  tags: [
    { name: 'Auth', description: 'Authentication endpoints' },
    { name: 'Admin - Users', description: 'User management (admin only)' },
    { name: 'Admin - Contacts', description: 'Contact submissions (admin only)' },
    { name: 'Admin - Settings', description: 'Site settings (admin only)' },
    { name: 'Admin - Translations', description: 'i18n translations (admin only)' },
    { name: 'Admin - System', description: 'System health & logs (admin only)' },
    { name: 'Portfolios', description: 'Portfolio management' },
    { name: 'Public', description: 'Public endpoints' },
    { name: 'User', description: 'Authenticated user endpoints' }
  ],
  paths: {
    // ═══════════════════════════════════════════════════════════════════════
    // AUTH
    // ═══════════════════════════════════════════════════════════════════════
    '/auth/login': {
      post: {
        tags: ['Auth'],
        summary: 'Login',
        description: 'Authenticate with email and password. Returns session cookie and CSRF token.',
        requestBody: {
          required: true,
          content: {
            'application/json': {
              schema: { $ref: '#/components/schemas/LoginRequest' }
            }
          }
        },
        responses: {
          200: {
            description: 'Login successful',
            content: {
              'application/json': {
                schema: { $ref: '#/components/schemas/LoginResponse' }
              }
            }
          },
          401: { description: 'Invalid credentials' },
          423: { description: 'Account locked' },
          429: { description: 'Rate limit exceeded' }
        }
      }
    },
    '/auth/logout': {
      post: {
        tags: ['Auth'],
        summary: 'Logout',
        description: 'End current session and clear cookies.',
        security: [{ cookieAuth: [] }],
        responses: {
          200: { description: 'Logout successful' }
        }
      }
    },
    '/auth/session': {
      get: {
        tags: ['Auth'],
        summary: 'Get current session',
        description: 'Returns current user info if authenticated.',
        responses: {
          200: {
            description: 'Session info',
            content: {
              'application/json': {
                schema: { $ref: '#/components/schemas/SessionResponse' }
              }
            }
          }
        }
      }
    },

    // ═══════════════════════════════════════════════════════════════════════
    // ADMIN - USERS
    // ═══════════════════════════════════════════════════════════════════════
    '/admin/users': {
      get: {
        tags: ['Admin - Users'],
        summary: 'List users',
        description: 'Get paginated list of users. Requires admin role.',
        security: [{ cookieAuth: [] }],
        parameters: [
          { $ref: '#/components/parameters/page' },
          { $ref: '#/components/parameters/limit' }
        ],
        responses: {
          200: {
            description: 'User list',
            content: {
              'application/json': {
                schema: { $ref: '#/components/schemas/UserListResponse' }
              }
            }
          },
          401: { description: 'Unauthorized' },
          403: { description: 'Forbidden - requires admin role' }
        }
      },
      post: {
        tags: ['Admin - Users'],
        summary: 'Create user',
        description: 'Create a new user. Requires admin role.',
        security: [{ cookieAuth: [] }, { csrfToken: [] }],
        requestBody: {
          required: true,
          content: {
            'application/json': {
              schema: { $ref: '#/components/schemas/CreateUserRequest' }
            }
          }
        },
        responses: {
          201: {
            description: 'User created',
            content: {
              'application/json': {
                schema: { $ref: '#/components/schemas/User' }
              }
            }
          },
          400: { description: 'Validation error' },
          401: { description: 'Unauthorized' },
          403: { description: 'Forbidden' },
          409: { description: 'Email already exists' }
        }
      }
    },
    '/admin/users/{id}': {
      put: {
        tags: ['Admin - Users'],
        summary: 'Update user',
        security: [{ cookieAuth: [] }, { csrfToken: [] }],
        parameters: [{ $ref: '#/components/parameters/userId' }],
        requestBody: {
          required: true,
          content: {
            'application/json': {
              schema: { $ref: '#/components/schemas/UpdateUserRequest' }
            }
          }
        },
        responses: {
          200: { description: 'User updated' },
          400: { description: 'Validation error' },
          401: { description: 'Unauthorized' },
          403: { description: 'Forbidden' },
          404: { description: 'User not found' }
        }
      },
      delete: {
        tags: ['Admin - Users'],
        summary: 'Delete user',
        security: [{ cookieAuth: [] }, { csrfToken: [] }],
        parameters: [{ $ref: '#/components/parameters/userId' }],
        responses: {
          200: { description: 'User deleted' },
          401: { description: 'Unauthorized' },
          403: { description: 'Forbidden - cannot delete self or higher role' },
          404: { description: 'User not found' }
        }
      }
    },

    // ═══════════════════════════════════════════════════════════════════════
    // ADMIN - CONTACTS
    // ═══════════════════════════════════════════════════════════════════════
    '/admin/contacts': {
      get: {
        tags: ['Admin - Contacts'],
        summary: 'List contact submissions',
        security: [{ cookieAuth: [] }],
        parameters: [
          { $ref: '#/components/parameters/page' },
          { $ref: '#/components/parameters/limit' },
          {
            name: 'unread',
            in: 'query',
            schema: { type: 'boolean' },
            description: 'Filter to unread only'
          }
        ],
        responses: {
          200: {
            description: 'Contact list',
            content: {
              'application/json': {
                schema: { $ref: '#/components/schemas/ContactListResponse' }
              }
            }
          },
          401: { description: 'Unauthorized' }
        }
      }
    },
    '/admin/contacts/{id}': {
      put: {
        tags: ['Admin - Contacts'],
        summary: 'Mark contact as read',
        security: [{ cookieAuth: [] }, { csrfToken: [] }],
        parameters: [{ $ref: '#/components/parameters/contactId' }],
        requestBody: {
          content: {
            'application/json': {
              schema: {
                type: 'object',
                properties: {
                  read: { type: 'boolean' }
                }
              }
            }
          }
        },
        responses: {
          200: { description: 'Contact updated' },
          404: { description: 'Contact not found' }
        }
      },
      delete: {
        tags: ['Admin - Contacts'],
        summary: 'Delete contact',
        security: [{ cookieAuth: [] }, { csrfToken: [] }],
        parameters: [{ $ref: '#/components/parameters/contactId' }],
        responses: {
          200: { description: 'Contact deleted' },
          404: { description: 'Contact not found' }
        }
      }
    },

    // ═══════════════════════════════════════════════════════════════════════
    // ADMIN - SETTINGS
    // ═══════════════════════════════════════════════════════════════════════
    '/admin/settings': {
      put: {
        tags: ['Admin - Settings'],
        summary: 'Update settings',
        security: [{ cookieAuth: [] }, { csrfToken: [] }],
        requestBody: {
          required: true,
          content: {
            'application/json': {
              schema: {
                type: 'object',
                additionalProperties: true,
                example: {
                  'contact.email': 'hello@example.com',
                  'contact.phone': '+1234567890'
                }
              }
            }
          }
        },
        responses: {
          200: { description: 'Settings updated' },
          401: { description: 'Unauthorized' }
        }
      }
    },
    '/admin/stats': {
      get: {
        tags: ['Admin - System'],
        summary: 'Get dashboard stats',
        security: [{ cookieAuth: [] }],
        responses: {
          200: {
            description: 'Dashboard statistics',
            content: {
              'application/json': {
                schema: { $ref: '#/components/schemas/StatsResponse' }
              }
            }
          }
        }
      }
    },

    // ═══════════════════════════════════════════════════════════════════════
    // ADMIN - TRANSLATIONS
    // ═══════════════════════════════════════════════════════════════════════
    '/admin/translations': {
      get: {
        tags: ['Admin - Translations'],
        summary: 'List translations',
        security: [{ cookieAuth: [] }],
        parameters: [
          {
            name: 'locale',
            in: 'query',
            schema: { type: 'string' },
            description: 'Filter by locale (e.g., en, ru, he)'
          }
        ],
        responses: {
          200: { description: 'Translation list' }
        }
      },
      post: {
        tags: ['Admin - Translations'],
        summary: 'Create/update translation',
        security: [{ cookieAuth: [] }, { csrfToken: [] }],
        requestBody: {
          required: true,
          content: {
            'application/json': {
              schema: { $ref: '#/components/schemas/TranslationRequest' }
            }
          }
        },
        responses: {
          200: { description: 'Translation saved' },
          400: { description: 'Validation error' }
        }
      }
    },
    '/admin/translations/{id}': {
      delete: {
        tags: ['Admin - Translations'],
        summary: 'Delete translation',
        security: [{ cookieAuth: [] }, { csrfToken: [] }],
        parameters: [
          {
            name: 'id',
            in: 'path',
            required: true,
            schema: { type: 'integer' }
          }
        ],
        responses: {
          200: { description: 'Translation deleted' },
          404: { description: 'Translation not found' }
        }
      }
    },

    // ═══════════════════════════════════════════════════════════════════════
    // ADMIN - SYSTEM
    // ═══════════════════════════════════════════════════════════════════════
    '/admin/health': {
      get: {
        tags: ['Admin - System'],
        summary: 'System health check',
        security: [{ cookieAuth: [] }],
        responses: {
          200: {
            description: 'Health status',
            content: {
              'application/json': {
                schema: { $ref: '#/components/schemas/HealthResponse' }
              }
            }
          }
        }
      }
    },
    '/admin/logs': {
      get: {
        tags: ['Admin - System'],
        summary: 'Get recent logs',
        security: [{ cookieAuth: [] }],
        parameters: [
          {
            name: 'limit',
            in: 'query',
            schema: { type: 'integer', default: 100 }
          },
          {
            name: 'level',
            in: 'query',
            schema: { type: 'integer' },
            description: 'Minimum log level (10=trace, 20=debug, 30=info, 40=warn, 50=error)'
          }
        ],
        responses: {
          200: { description: 'Log entries' }
        }
      }
    },
    '/admin/audit-logs': {
      get: {
        tags: ['Admin - System'],
        summary: 'Get audit logs',
        security: [{ cookieAuth: [] }],
        parameters: [
          { $ref: '#/components/parameters/page' },
          { $ref: '#/components/parameters/limit' }
        ],
        responses: {
          200: { description: 'Audit log entries' }
        }
      }
    },

    // ═══════════════════════════════════════════════════════════════════════
    // PORTFOLIOS
    // ═══════════════════════════════════════════════════════════════════════
    '/portfolios': {
      get: {
        tags: ['Portfolios'],
        summary: 'List portfolios',
        parameters: [
          {
            name: 'published',
            in: 'query',
            schema: { type: 'boolean' },
            description: 'Filter by published status'
          }
        ],
        responses: {
          200: {
            description: 'Portfolio list',
            content: {
              'application/json': {
                schema: { $ref: '#/components/schemas/PortfolioListResponse' }
              }
            }
          }
        }
      },
      post: {
        tags: ['Portfolios'],
        summary: 'Create portfolio',
        security: [{ cookieAuth: [] }, { csrfToken: [] }],
        requestBody: {
          required: true,
          content: {
            'application/json': {
              schema: { $ref: '#/components/schemas/CreatePortfolioRequest' }
            }
          }
        },
        responses: {
          201: { description: 'Portfolio created' },
          400: { description: 'Validation error' },
          401: { description: 'Unauthorized' }
        }
      }
    },
    '/portfolios/{id}': {
      get: {
        tags: ['Portfolios'],
        summary: 'Get portfolio by ID or slug',
        parameters: [{ $ref: '#/components/parameters/portfolioId' }],
        responses: {
          200: {
            description: 'Portfolio details',
            content: {
              'application/json': {
                schema: { $ref: '#/components/schemas/Portfolio' }
              }
            }
          },
          404: { description: 'Portfolio not found' }
        }
      },
      put: {
        tags: ['Portfolios'],
        summary: 'Update portfolio',
        security: [{ cookieAuth: [] }, { csrfToken: [] }],
        parameters: [{ $ref: '#/components/parameters/portfolioId' }],
        requestBody: {
          required: true,
          content: {
            'application/json': {
              schema: { $ref: '#/components/schemas/UpdatePortfolioRequest' }
            }
          }
        },
        responses: {
          200: { description: 'Portfolio updated' },
          404: { description: 'Portfolio not found' }
        }
      },
      delete: {
        tags: ['Portfolios'],
        summary: 'Delete portfolio',
        security: [{ cookieAuth: [] }, { csrfToken: [] }],
        parameters: [{ $ref: '#/components/parameters/portfolioId' }],
        responses: {
          200: { description: 'Portfolio deleted' },
          404: { description: 'Portfolio not found' }
        }
      }
    },
    '/portfolios/{id}/items': {
      get: {
        tags: ['Portfolios'],
        summary: 'List portfolio items',
        parameters: [{ $ref: '#/components/parameters/portfolioId' }],
        responses: {
          200: { description: 'Item list' }
        }
      },
      post: {
        tags: ['Portfolios'],
        summary: 'Add portfolio item',
        security: [{ cookieAuth: [] }, { csrfToken: [] }],
        parameters: [{ $ref: '#/components/parameters/portfolioId' }],
        requestBody: {
          required: true,
          content: {
            'application/json': {
              schema: { $ref: '#/components/schemas/CreatePortfolioItemRequest' }
            }
          }
        },
        responses: {
          201: { description: 'Item created' }
        }
      }
    },
    '/portfolios/{id}/items/{itemId}': {
      put: {
        tags: ['Portfolios'],
        summary: 'Update portfolio item',
        security: [{ cookieAuth: [] }, { csrfToken: [] }],
        parameters: [
          { $ref: '#/components/parameters/portfolioId' },
          { $ref: '#/components/parameters/itemId' }
        ],
        responses: {
          200: { description: 'Item updated' }
        }
      },
      delete: {
        tags: ['Portfolios'],
        summary: 'Delete portfolio item',
        security: [{ cookieAuth: [] }, { csrfToken: [] }],
        parameters: [
          { $ref: '#/components/parameters/portfolioId' },
          { $ref: '#/components/parameters/itemId' }
        ],
        responses: {
          200: { description: 'Item deleted' }
        }
      }
    },
    '/portfolios/{id}/items/reorder': {
      post: {
        tags: ['Portfolios'],
        summary: 'Reorder portfolio items',
        security: [{ cookieAuth: [] }, { csrfToken: [] }],
        parameters: [{ $ref: '#/components/parameters/portfolioId' }],
        requestBody: {
          required: true,
          content: {
            'application/json': {
              schema: {
                type: 'object',
                properties: {
                  itemIds: {
                    type: 'array',
                    items: { type: 'integer' },
                    description: 'Item IDs in new order'
                  }
                }
              }
            }
          }
        },
        responses: {
          200: { description: 'Items reordered' }
        }
      }
    },

    // ═══════════════════════════════════════════════════════════════════════
    // PUBLIC
    // ═══════════════════════════════════════════════════════════════════════
    '/contact/submit': {
      post: {
        tags: ['Public'],
        summary: 'Submit contact form',
        description: 'Rate limited: 5 submissions per hour per IP',
        requestBody: {
          required: true,
          content: {
            'application/json': {
              schema: { $ref: '#/components/schemas/ContactRequest' }
            }
          }
        },
        responses: {
          200: { description: 'Message sent' },
          400: { description: 'Validation error' },
          429: { description: 'Rate limit exceeded' }
        }
      }
    },
    '/settings': {
      get: {
        tags: ['Public'],
        summary: 'Get public settings',
        responses: {
          200: {
            description: 'Public settings',
            content: {
              'application/json': {
                schema: {
                  type: 'object',
                  additionalProperties: true
                }
              }
            }
          }
        }
      }
    },
    '/health': {
      get: {
        tags: ['Public'],
        summary: 'Public health check',
        responses: {
          200: {
            description: 'Service healthy',
            content: {
              'application/json': {
                schema: {
                  type: 'object',
                  properties: {
                    status: { type: 'string', example: 'ok' },
                    timestamp: { type: 'string', format: 'date-time' }
                  }
                }
              }
            }
          }
        }
      }
    },
    '/i18n/{locale}': {
      get: {
        tags: ['Public'],
        summary: 'Get translations for locale',
        parameters: [
          {
            name: 'locale',
            in: 'path',
            required: true,
            schema: { type: 'string' },
            example: 'en'
          }
        ],
        responses: {
          200: { description: 'Translation messages' }
        }
      }
    },

    // ═══════════════════════════════════════════════════════════════════════
    // USER
    // ═══════════════════════════════════════════════════════════════════════
    '/user/change-password': {
      put: {
        tags: ['User'],
        summary: 'Change own password',
        security: [{ cookieAuth: [] }, { csrfToken: [] }],
        requestBody: {
          required: true,
          content: {
            'application/json': {
              schema: { $ref: '#/components/schemas/ChangePasswordRequest' }
            }
          }
        },
        responses: {
          200: { description: 'Password changed' },
          400: { description: 'Validation error' },
          401: { description: 'Current password incorrect' }
        }
      }
    },

    // ═══════════════════════════════════════════════════════════════════════
    // UPLOADS
    // ═══════════════════════════════════════════════════════════════════════
    '/upload/image': {
      post: {
        tags: ['Portfolios'],
        summary: 'Upload image',
        security: [{ cookieAuth: [] }],
        requestBody: {
          required: true,
          content: {
            'multipart/form-data': {
              schema: {
                type: 'object',
                properties: {
                  file: {
                    type: 'string',
                    format: 'binary'
                  }
                }
              }
            }
          }
        },
        responses: {
          200: {
            description: 'Upload successful',
            content: {
              'application/json': {
                schema: {
                  type: 'object',
                  properties: {
                    url: { type: 'string' },
                    thumbnailUrl: { type: 'string' }
                  }
                }
              }
            }
          }
        }
      }
    },
    '/upload/video': {
      post: {
        tags: ['Portfolios'],
        summary: 'Upload video',
        security: [{ cookieAuth: [] }],
        requestBody: {
          required: true,
          content: {
            'multipart/form-data': {
              schema: {
                type: 'object',
                properties: {
                  file: {
                    type: 'string',
                    format: 'binary'
                  }
                }
              }
            }
          }
        },
        responses: {
          200: {
            description: 'Upload successful',
            content: {
              'application/json': {
                schema: {
                  type: 'object',
                  properties: {
                    url: { type: 'string' },
                    thumbnailUrl: { type: 'string' }
                  }
                }
              }
            }
          }
        }
      }
    }
  },
  components: {
    securitySchemes: {
      cookieAuth: {
        type: 'apiKey',
        in: 'cookie',
        name: 'pm-session',
        description: 'Session cookie obtained from login'
      },
      csrfToken: {
        type: 'apiKey',
        in: 'header',
        name: 'x-csrf-token',
        description: 'CSRF token from login response'
      }
    },
    parameters: {
      page: {
        name: 'page',
        in: 'query',
        schema: { type: 'integer', default: 1, minimum: 1 },
        description: 'Page number'
      },
      limit: {
        name: 'limit',
        in: 'query',
        schema: { type: 'integer', default: 20, minimum: 1, maximum: 100 },
        description: 'Items per page'
      },
      userId: {
        name: 'id',
        in: 'path',
        required: true,
        schema: { type: 'integer' },
        description: 'User ID'
      },
      contactId: {
        name: 'id',
        in: 'path',
        required: true,
        schema: { type: 'integer' },
        description: 'Contact submission ID'
      },
      portfolioId: {
        name: 'id',
        in: 'path',
        required: true,
        schema: { oneOf: [{ type: 'integer' }, { type: 'string' }] },
        description: 'Portfolio ID or slug'
      },
      itemId: {
        name: 'itemId',
        in: 'path',
        required: true,
        schema: { type: 'integer' },
        description: 'Portfolio item ID'
      }
    },
    schemas: {
      // Auth
      LoginRequest: {
        type: 'object',
        required: ['email', 'password'],
        properties: {
          email: { type: 'string', format: 'email' },
          password: { type: 'string', minLength: 1 },
          rememberMe: { type: 'boolean', default: false }
        }
      },
      LoginResponse: {
        type: 'object',
        properties: {
          success: { type: 'boolean' },
          user: { $ref: '#/components/schemas/User' },
          csrfToken: { type: 'string' }
        }
      },
      SessionResponse: {
        type: 'object',
        properties: {
          user: { $ref: '#/components/schemas/User' },
          csrfToken: { type: 'string' }
        }
      },

      // User
      User: {
        type: 'object',
        properties: {
          id: { type: 'integer' },
          email: { type: 'string', format: 'email' },
          name: { type: 'string' },
          role: { type: 'string', enum: ['master', 'admin', 'editor'] },
          createdAt: { type: 'string', format: 'date-time' }
        }
      },
      CreateUserRequest: {
        type: 'object',
        required: ['email', 'password', 'role'],
        properties: {
          email: { type: 'string', format: 'email' },
          password: { type: 'string', minLength: 8 },
          name: { type: 'string', maxLength: 100 },
          role: { type: 'string', enum: ['master', 'admin', 'editor'] }
        }
      },
      UpdateUserRequest: {
        type: 'object',
        properties: {
          email: { type: 'string', format: 'email' },
          password: { type: 'string', minLength: 8 },
          name: { type: 'string', maxLength: 100 },
          role: { type: 'string', enum: ['master', 'admin', 'editor'] }
        }
      },
      UserListResponse: {
        type: 'object',
        properties: {
          items: {
            type: 'array',
            items: { $ref: '#/components/schemas/User' }
          },
          meta: { $ref: '#/components/schemas/PaginationMeta' }
        }
      },
      ChangePasswordRequest: {
        type: 'object',
        required: ['currentPassword', 'newPassword'],
        properties: {
          currentPassword: { type: 'string' },
          newPassword: { type: 'string', minLength: 8 }
        }
      },

      // Contact
      ContactRequest: {
        type: 'object',
        required: ['name', 'email', 'message'],
        properties: {
          name: { type: 'string', minLength: 2, maxLength: 100 },
          email: { type: 'string', format: 'email', maxLength: 255 },
          phone: { type: 'string', maxLength: 30 },
          subject: { type: 'string', maxLength: 200 },
          message: { type: 'string', minLength: 10, maxLength: 5000 }
        }
      },
      Contact: {
        type: 'object',
        properties: {
          id: { type: 'integer' },
          name: { type: 'string' },
          email: { type: 'string' },
          phone: { type: 'string' },
          subject: { type: 'string' },
          message: { type: 'string' },
          read: { type: 'boolean' },
          createdAt: { type: 'string', format: 'date-time' }
        }
      },
      ContactListResponse: {
        type: 'object',
        properties: {
          items: {
            type: 'array',
            items: { $ref: '#/components/schemas/Contact' }
          },
          meta: { $ref: '#/components/schemas/PaginationMeta' },
          unreadCount: { type: 'integer' }
        }
      },

      // Portfolio
      Portfolio: {
        type: 'object',
        properties: {
          id: { type: 'integer' },
          slug: { type: 'string' },
          name: { type: 'string' },
          description: { type: 'string' },
          type: { type: 'string', enum: ['gallery', 'case_study'] },
          coverImageUrl: { type: 'string' },
          coverThumbnailUrl: { type: 'string' },
          order: { type: 'integer' },
          published: { type: 'boolean' },
          items: {
            type: 'array',
            items: { $ref: '#/components/schemas/PortfolioItem' }
          }
        }
      },
      PortfolioItem: {
        type: 'object',
        properties: {
          id: { type: 'integer' },
          portfolioId: { type: 'integer' },
          itemType: { type: 'string', enum: ['image', 'video', 'link', 'case_study'] },
          mediaUrl: { type: 'string' },
          thumbnailUrl: { type: 'string' },
          caption: { type: 'string' },
          slug: { type: 'string' },
          title: { type: 'string' },
          description: { type: 'string' },
          content: { type: 'string' },
          category: { type: 'string' },
          tags: { type: 'array', items: { type: 'string' } },
          order: { type: 'integer' }
        }
      },
      CreatePortfolioRequest: {
        type: 'object',
        required: ['slug', 'name'],
        properties: {
          slug: { type: 'string', pattern: '^[a-z0-9-]+$' },
          name: { type: 'string' },
          description: { type: 'string' },
          type: { type: 'string', enum: ['gallery', 'case_study'], default: 'gallery' },
          published: { type: 'boolean', default: false }
        }
      },
      UpdatePortfolioRequest: {
        type: 'object',
        properties: {
          slug: { type: 'string', pattern: '^[a-z0-9-]+$' },
          name: { type: 'string' },
          description: { type: 'string' },
          type: { type: 'string', enum: ['gallery', 'case_study'] },
          coverImageUrl: { type: 'string' },
          order: { type: 'integer' },
          published: { type: 'boolean' }
        }
      },
      CreatePortfolioItemRequest: {
        type: 'object',
        required: ['itemType'],
        properties: {
          itemType: { type: 'string', enum: ['image', 'video', 'link', 'case_study'] },
          mediaUrl: { type: 'string' },
          thumbnailUrl: { type: 'string' },
          caption: { type: 'string' },
          slug: { type: 'string' },
          title: { type: 'string' },
          description: { type: 'string' },
          content: { type: 'string' },
          category: { type: 'string' },
          tags: { type: 'array', items: { type: 'string' } }
        }
      },
      PortfolioListResponse: {
        type: 'object',
        properties: {
          items: {
            type: 'array',
            items: { $ref: '#/components/schemas/Portfolio' }
          }
        }
      },

      // Translations
      TranslationRequest: {
        type: 'object',
        required: ['locale', 'key', 'value'],
        properties: {
          locale: { type: 'string', example: 'en' },
          key: { type: 'string', example: 'nav.home' },
          value: { type: 'string', example: 'Home' }
        }
      },

      // System
      StatsResponse: {
        type: 'object',
        properties: {
          users: { type: 'integer' },
          portfolios: { type: 'integer' },
          contacts: { type: 'integer' },
          unreadContacts: { type: 'integer' }
        }
      },
      HealthResponse: {
        type: 'object',
        properties: {
          status: { type: 'string', enum: ['ok', 'degraded', 'error'] },
          database: { type: 'boolean' },
          uptime: { type: 'number' },
          memory: {
            type: 'object',
            properties: {
              used: { type: 'number' },
              total: { type: 'number' }
            }
          },
          timestamp: { type: 'string', format: 'date-time' }
        }
      },

      // Pagination
      PaginationMeta: {
        type: 'object',
        properties: {
          page: { type: 'integer' },
          limit: { type: 'integer' },
          total: { type: 'integer' },
          totalPages: { type: 'integer' },
          hasMore: { type: 'boolean' }
        }
      }
    }
  }
}
