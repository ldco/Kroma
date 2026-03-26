<script setup lang="ts">
/**
 * Admin Blog Page
 *
 * Manage blog posts, categories, and tags with multi-language support.
 * All languages are equal - edited in the same modal with tabs.
 *
 * CSS Dependencies (see CSS-COMPONENT-MAP.md):
 * - ui/overlays/modal.css: .modal-backdrop, .modal, .modal--lg, .modal-header, .modal-body, .modal-footer
 * - ui/forms/inputs.css: .input, .form-group, .form-label, .form-row--2col, .form-checkbox, .input-row, .form-divider
 * - ui/content/tabs.css: .tabs, .tabs--underline, .tab, .is-active, .tab__indicator
 * - ui/content/cards.css: .card, .card-body, .card-actions
 * - ui/admin/pages.css: .blog-admin-list, .blog-admin-item__*, .simple-list
 */
import IconPlus from '~icons/tabler/plus'
import IconEdit from '~icons/tabler/pencil'
import IconTrash from '~icons/tabler/trash'
import IconX from '~icons/tabler/x'
import IconUpload from '~icons/tabler/upload'
import IconEye from '~icons/tabler/eye'
import IconCalendar from '~icons/tabler/calendar'
import IconTag from '~icons/tabler/tag'
import IconFolder from '~icons/tabler/folder'
import config from '~/puppet-master.config'
import AppImage from '~/components/atoms/AppImage.vue'

interface BlogCategory {
  id: number
  slug: string
  name: string
  translations: Record<string, { name: string }>
}

interface BlogTag {
  id: number
  slug: string
  name: string
}

interface BlogPost {
  id: number
  slug: string
  coverImage: string | null
  categoryId: number | null
  category: BlogCategory | null
  tags: BlogTag[]
  status: 'draft' | 'published' | 'archived'
  publishedAt: string | null
  views: number
  readTime: number | null
  translations: Record<string, { title: string | null; excerpt: string | null; content: string | null }>
  createdAt: string
}

definePageMeta({
  layout: 'admin',
  middleware: 'auth',
  pageTransition: false
})

const { t } = useI18n()

useHead({
  title: () => `${t('admin.blog')} | Admin`
})

const { confirm } = useConfirm()
const { toast } = useToast()

// SSR-safe config access with defensive defaults
const localesConfig = computed(() => config?.locales ?? [])
const defaultLocale = computed(() => config?.defaultLocale || 'en')

// Available locales from config
const locales = localesConfig.value.map(l => l.code)

// Fetch data
const headers = useRequestHeaders(['cookie'])

const {
  data: posts,
  pending: loadingPosts,
  refresh: refreshPosts
} = await useAsyncData(
  'admin-blog-posts',
  () => apiFetch<BlogPost[]>('/api/admin/blog/posts', { headers }),
  { default: () => [] }
)

const { data: categories, refresh: refreshCategories } = await useAsyncData(
  'admin-blog-categories',
  () => apiFetch<BlogCategory[]>('/api/admin/blog/categories', { headers }),
  { default: () => [] }
)
const { data: tags, refresh: refreshTags } = await useAsyncData(
  'admin-blog-tags',
  () => apiFetch<BlogTag[]>('/api/admin/blog/tags', { headers }),
  { default: () => [] }
)

// Tab state
const activeTab = ref<'posts' | 'categories' | 'tags'>('posts')

// Post modal state
const showPostModal = ref(false)
const editingPost = ref<BlogPost | null>(null)
const saving = ref(false)
const activeLocaleTab = ref(defaultLocale)

// Initialize empty translations for all locales
function createEmptyTranslations() {
  const trans: Record<string, { title: string; excerpt: string; content: string }> = {}
  for (const locale of locales) {
    trans[locale] = { title: '', excerpt: '', content: '' }
  }
  return trans
}

const postForm = reactive({
  slug: '',
  coverImage: '',
  categoryId: null as number | null,
  tagIds: [] as number[],
  status: 'draft' as 'draft' | 'published' | 'archived',
  publishedAt: '',
  readTime: null as number | null,
  translations: createEmptyTranslations()
})

// Category modal state
const showCategoryModal = ref(false)
const editingCategory = ref<BlogCategory | null>(null)
const activeCategoryLocaleTab = ref(defaultLocale)
const categoryForm = reactive({
  slug: '',
  translations: {} as Record<string, { name: string }>
})

// Tag modal state
const showTagModal = ref(false)
const editingTag = ref<BlogTag | null>(null)
const tagForm = reactive({
  slug: '',
  name: ''
})

// Reset forms
function resetPostForm() {
  postForm.slug = ''
  postForm.coverImage = ''
  postForm.categoryId = null
  postForm.tagIds = []
  postForm.status = 'draft'
  postForm.publishedAt = ''
  postForm.readTime = null
  postForm.translations = createEmptyTranslations()
  activeLocaleTab.value = defaultLocale
}

function resetCategoryForm() {
  categoryForm.slug = ''
  categoryForm.translations = {}
  for (const locale of locales) {
    categoryForm.translations[locale] = { name: '' }
  }
  activeCategoryLocaleTab.value = defaultLocale
}

function resetTagForm() {
  tagForm.slug = ''
  tagForm.name = ''
}

// Post CRUD
function openCreatePost() {
  editingPost.value = null
  resetPostForm()
  showPostModal.value = true
}

function openEditPost(post: BlogPost) {
  editingPost.value = post
  postForm.slug = post.slug
  postForm.coverImage = post.coverImage || ''
  postForm.categoryId = post.categoryId
  postForm.tagIds = post.tags?.map(t => t.id) || []
  postForm.status = post.status
  postForm.publishedAt = post.publishedAt?.split('T')[0] || ''
  postForm.readTime = post.readTime

  // Load all translations
  postForm.translations = createEmptyTranslations()
  for (const locale of locales) {
    const trans = post.translations?.[locale]
    if (trans) {
      postForm.translations[locale] = {
        title: trans.title || '',
        excerpt: trans.excerpt || '',
        content: trans.content || ''
      }
    }
  }

  activeLocaleTab.value = defaultLocale
  showPostModal.value = true
}

// Auto-generate slug from first non-empty title
function generateSlug() {
  if (!editingPost.value && !postForm.slug) {
    // Find first non-empty title
    for (const locale of locales) {
      const title = postForm.translations[locale]?.title
      if (title) {
        postForm.slug = title
          .toLowerCase()
          .replace(/[^a-z0-9]+/g, '-')
          .replace(/^-|-$/g, '')
        break
      }
    }
  }
}

async function savePost() {
  saving.value = true

  try {
    const payload = {
      slug: postForm.slug,
      coverImage: postForm.coverImage || null,
      categoryId: postForm.categoryId,
      tagIds: postForm.tagIds,
      status: postForm.status,
      publishedAt: postForm.publishedAt || null,
      readTime: postForm.readTime,
      translations: postForm.translations
    }

    if (editingPost.value) {
      await $fetch(`/api/admin/blog/posts/${editingPost.value.id}`, {
        method: 'PUT',
        body: payload
      })
    } else {
      await $fetch('/api/admin/blog/posts', {
        method: 'POST',
        body: payload
      })
    }
    toast.success(t('common.saved'))
    showPostModal.value = false
    refreshPosts()
  } catch (e: unknown) {
    const error = e as { data?: { message?: string } }
    toast.error(error.data?.message || t('common.error'))
  } finally {
    saving.value = false
  }
}

async function deletePost(post: BlogPost) {
  const confirmed = await confirm(t('admin.confirmDelete'), {
    title: t('common.delete'),
    confirmText: t('common.delete'),
    variant: 'danger'
  })

  if (!confirmed) return

  try {
    await $fetch(`/api/admin/blog/posts/${post.id}`, { method: 'DELETE' })
    toast.success(t('common.deleted'))
    refreshPosts()
  } catch (e: unknown) {
    const error = e as { data?: { message?: string } }
    toast.error(error.data?.message || t('common.error'))
  }
}

// Category CRUD
function openCreateCategory() {
  editingCategory.value = null
  resetCategoryForm()
  showCategoryModal.value = true
}

function openEditCategory(category: BlogCategory) {
  editingCategory.value = category
  categoryForm.slug = category.slug

  // Load all translations
  categoryForm.translations = {}
  for (const locale of locales) {
    const trans = category.translations?.[locale]
    categoryForm.translations[locale] = { name: trans?.name || '' }
  }

  activeCategoryLocaleTab.value = defaultLocale
  showCategoryModal.value = true
}

function generateCategorySlug() {
  if (!editingCategory.value && !categoryForm.slug) {
    for (const locale of locales) {
      const name = categoryForm.translations[locale]?.name
      if (name) {
        categoryForm.slug = name
          .toLowerCase()
          .replace(/[^a-z0-9]+/g, '-')
          .replace(/^-|-$/g, '')
        break
      }
    }
  }
}

async function saveCategory() {
  saving.value = true

  try {
    const payload = {
      slug: categoryForm.slug,
      translations: categoryForm.translations
    }

    if (editingCategory.value) {
      await $fetch(`/api/admin/blog/categories/${editingCategory.value.id}`, {
        method: 'PUT',
        body: payload
      })
    } else {
      await $fetch('/api/admin/blog/categories', {
        method: 'POST',
        body: payload
      })
    }
    toast.success(t('common.saved'))
    showCategoryModal.value = false
    refreshCategories()
  } catch (e: unknown) {
    const error = e as { data?: { message?: string } }
    toast.error(error.data?.message || t('common.error'))
  } finally {
    saving.value = false
  }
}

async function deleteCategory(category: BlogCategory) {
  const confirmed = await confirm(t('admin.confirmDelete'), {
    title: t('common.delete'),
    confirmText: t('common.delete'),
    variant: 'danger'
  })

  if (!confirmed) return

  try {
    await $fetch(`/api/admin/blog/categories/${category.id}`, { method: 'DELETE' })
    toast.success(t('common.deleted'))
    refreshCategories()
  } catch (e: unknown) {
    const error = e as { data?: { message?: string } }
    toast.error(error.data?.message || t('common.error'))
  }
}

// Tag CRUD
function openCreateTag() {
  editingTag.value = null
  resetTagForm()
  showTagModal.value = true
}

function openEditTag(tag: BlogTag) {
  editingTag.value = tag
  tagForm.slug = tag.slug
  tagForm.name = tag.name
  showTagModal.value = true
}

function generateTagSlug() {
  if (!editingTag.value && tagForm.name && !tagForm.slug) {
    tagForm.slug = tagForm.name
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/^-|-$/g, '')
  }
}

async function saveTag() {
  saving.value = true

  try {
    if (editingTag.value) {
      await $fetch(`/api/admin/blog/tags/${editingTag.value.id}`, {
        method: 'PUT',
        body: tagForm
      })
    } else {
      await $fetch('/api/admin/blog/tags', {
        method: 'POST',
        body: tagForm
      })
    }
    toast.success(t('common.saved'))
    showTagModal.value = false
    refreshTags()
  } catch (e: unknown) {
    const error = e as { data?: { message?: string } }
    toast.error(error.data?.message || t('common.error'))
  } finally {
    saving.value = false
  }
}

async function deleteTag(tag: BlogTag) {
  const confirmed = await confirm(t('admin.confirmDelete'), {
    title: t('common.delete'),
    confirmText: t('common.delete'),
    variant: 'danger'
  })

  if (!confirmed) return

  try {
    await $fetch(`/api/admin/blog/tags/${tag.id}`, { method: 'DELETE' })
    toast.success(t('common.deleted'))
    refreshTags()
  } catch (e: unknown) {
    const error = e as { data?: { message?: string } }
    toast.error(error.data?.message || t('common.error'))
  }
}

// Helpers
function getLocaleName(code: string): string {
  const locale = localesConfig.value.find(l => l.code === code)
  return locale?.name || code.toUpperCase()
}

function getPostTitle(post: BlogPost): string {
  // Get first available title from translations
  for (const locale of locales) {
    const title = post.translations?.[locale]?.title
    if (title) return title
  }
  return post.slug
}

function getPostExcerpt(post: BlogPost): string | null {
  for (const locale of locales) {
    const excerpt = post.translations?.[locale]?.excerpt
    if (excerpt) return excerpt
  }
  return null
}

function getCategoryName(category: BlogCategory): string {
  for (const locale of locales) {
    const name = category.translations?.[locale]?.name
    if (name) return name
  }
  return category.slug
}

function hasAllTranslations(post: BlogPost): boolean {
  return locales.every(locale => post.translations?.[locale]?.title)
}

function getMissingTranslations(post: BlogPost): string[] {
  return locales.filter(locale => !post.translations?.[locale]?.title)
}

function getStatusBadgeClass(status: string) {
  switch (status) {
    case 'published':
      return 'badge-success'
    case 'archived':
      return 'badge-secondary'
    default:
      return 'badge-warning'
  }
}

function formatDate(dateStr: string | null) {
  if (!dateStr) return '-'
  return new Date(dateStr).toLocaleDateString()
}

// Cover image upload
const uploadingCover = ref(false)
async function uploadCoverImage(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return

  uploadingCover.value = true
  try {
    const formData = new FormData()
    formData.append('file', file)

    const result = await $fetch<{ url: string }>('/api/upload/image', {
      method: 'POST',
      body: formData
    })

    postForm.coverImage = result.url
    toast.success(t('common.uploaded'))
  } catch (e: unknown) {
    const error = e as { data?: { message?: string } }
    toast.error(error.data?.message || t('common.error'))
  } finally {
    uploadingCover.value = false
    input.value = ''
  }
}
</script>

<template>
  <div class="admin-blog">
    <div class="page-header">
      <h1 class="page-title">{{ t('admin.blog') }}</h1>
      <button class="btn btn-primary" @click="activeTab === 'posts' ? openCreatePost() : activeTab === 'categories' ? openCreateCategory() : openCreateTag()">
        <IconPlus />
        {{ t('admin.addItem') }}
      </button>
    </div>

    <!-- Tabs -->
    <div class="tabs tabs--underline mb-6">
      <button
        class="tab"
        :class="{ 'is-active': activeTab === 'posts' }"
        @click="activeTab = 'posts'"
      >
        {{ t('admin.posts') }}
      </button>
      <button
        class="tab"
        :class="{ 'is-active': activeTab === 'categories' }"
        @click="activeTab = 'categories'"
      >
        {{ t('admin.categories') }}
      </button>
      <button
        class="tab"
        :class="{ 'is-active': activeTab === 'tags' }"
        @click="activeTab = 'tags'"
      >
        {{ t('admin.tags') }}
      </button>
    </div>

    <!-- Posts Tab -->
    <div v-if="activeTab === 'posts'">
      <div v-if="loadingPosts" class="loading-state">{{ t('common.loading') }}</div>

      <div v-else-if="!posts?.length" class="empty-state">
        <p>{{ t('admin.noItems') }}</p>
      </div>

      <div v-else class="blog-admin-list">
        <div
          v-for="post in posts"
          :key="post.id"
          class="blog-admin-item card"
        >
          <div class="card-body">
            <div class="blog-admin-item__header">
              <div v-if="post.coverImage" class="blog-admin-item__cover">
                <AppImage :src="post.coverImage" :alt="getPostTitle(post)" />
              </div>
              <div class="blog-admin-item__info">
                <h3 class="blog-admin-item__title">{{ getPostTitle(post) }}</h3>
                <p v-if="getPostExcerpt(post)" class="blog-admin-item__excerpt">{{ getPostExcerpt(post) }}</p>
                <div class="blog-admin-item__meta">
                  <span :class="['badge', getStatusBadgeClass(post.status)]">
                    {{ t(`admin.${post.status || 'draft'}`) }}
                  </span>
                  <span v-if="post.category" class="blog-admin-item__category">
                    <IconFolder class="icon-sm" />
                    {{ getCategoryName(post.category) }}
                  </span>
                  <span class="blog-admin-item__date">
                    <IconCalendar class="icon-sm" />
                    {{ formatDate(post.publishedAt || post.createdAt) }}
                  </span>
                  <span class="blog-admin-item__views">
                    <IconEye class="icon-sm" />
                    {{ post.views }}
                  </span>
                </div>
                <div v-if="post.tags?.length" class="blog-admin-item__tags">
                  <span v-for="tag in post.tags" :key="tag.id" class="badge badge-secondary">
                    {{ tag.name }}
                  </span>
                </div>
              </div>
            </div>

            <!-- Translation status indicator -->
            <div v-if="!hasAllTranslations(post)" class="blog-admin-item__translations">
              <span class="text-warning text-sm">
                {{ t('admin.missingTranslations') }}: {{ getMissingTranslations(post).map(l => l.toUpperCase()).join(', ') }}
              </span>
            </div>
          </div>

          <div class="card-actions">
            <button class="btn btn-sm btn-secondary" @click="openEditPost(post)">
              <IconEdit /> {{ t('common.edit') }}
            </button>
            <button class="btn btn-sm btn-ghost text-error" @click="deletePost(post)">
              <IconTrash />
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Categories Tab -->
    <div v-if="activeTab === 'categories'">
      <div v-if="!categories?.length" class="empty-state">
        <p>{{ t('admin.noItems') }}</p>
      </div>

      <div v-else class="simple-list">
        <div
          v-for="category in categories"
          :key="category.id"
          class="simple-list__item"
        >
          <div class="simple-list__content">
            <IconFolder class="icon" />
            <span class="simple-list__name">{{ getCategoryName(category) }}</span>
            <span class="simple-list__slug text-secondary">{{ category.slug }}</span>
          </div>
          <div class="simple-list__actions">
            <button class="btn btn-sm btn-ghost" @click="openEditCategory(category)">
              <IconEdit />
            </button>
            <button class="btn btn-sm btn-ghost text-error" @click="deleteCategory(category)">
              <IconTrash />
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Tags Tab -->
    <div v-if="activeTab === 'tags'">
      <div v-if="!tags?.length" class="empty-state">
        <p>{{ t('admin.noItems') }}</p>
      </div>

      <div v-else class="simple-list">
        <div
          v-for="tag in tags"
          :key="tag.id"
          class="simple-list__item"
        >
          <div class="simple-list__content">
            <IconTag class="icon" />
            <span class="simple-list__name">{{ tag.name }}</span>
            <span class="simple-list__slug text-secondary">{{ tag.slug }}</span>
          </div>
          <div class="simple-list__actions">
            <button class="btn btn-sm btn-ghost" @click="openEditTag(tag)">
              <IconEdit />
            </button>
            <button class="btn btn-sm btn-ghost text-error" @click="deleteTag(tag)">
              <IconTrash />
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Post Edit Modal with Language Tabs -->
    <Teleport to="body">
      <div v-if="showPostModal" class="modal-backdrop" @click.self="showPostModal = false">
        <div class="modal modal--lg">
          <header class="modal-header">
            <h2>{{ editingPost ? t('common.edit') : t('admin.addItem') }}</h2>
            <button class="btn btn-ghost btn-sm" @click="showPostModal = false">
              <IconX />
            </button>
          </header>

          <form class="modal-body" @submit.prevent="savePost">
            <!-- Non-translatable fields -->
            <div class="form-row form-row--2col">
              <div class="form-group">
                <label class="form-label" for="post-slug">{{ t('admin.slug') }} *</label>
                <input
                  id="post-slug"
                  v-model="postForm.slug"
                  type="text"
                  class="input"
                  required
                  pattern="[a-z0-9-]+"
                />
              </div>

              <div class="form-group">
                <label class="form-label" for="post-category">{{ t('admin.category') }}</label>
                <select id="post-category" v-model="postForm.categoryId" class="input">
                  <option :value="null">{{ t('common.none') }}</option>
                  <option v-for="cat in categories" :key="cat.id" :value="cat.id">
                    {{ getCategoryName(cat) }}
                  </option>
                </select>
              </div>

              <div class="form-group">
                <label class="form-label" for="post-status">{{ t('admin.status') }}</label>
                <select id="post-status" v-model="postForm.status" class="input">
                  <option value="draft">{{ t('common.draft') }}</option>
                  <option value="published">{{ t('admin.published') }}</option>
                  <option value="archived">{{ t('admin.archived') }}</option>
                </select>
              </div>

              <div class="form-group">
                <label class="form-label" for="post-published">{{ t('admin.publishedAt') }}</label>
                <input
                  id="post-published"
                  v-model="postForm.publishedAt"
                  type="date"
                  class="input"
                />
              </div>

              <div class="form-group">
                <label class="form-label" for="post-readtime">{{ t('admin.readTime') }}</label>
                <input
                  id="post-readtime"
                  v-model.number="postForm.readTime"
                  type="number"
                  class="input"
                  min="1"
                  placeholder="5"
                />
              </div>
            </div>

            <!-- Tags -->
            <div class="form-group">
              <label class="form-label">{{ t('admin.tags') }}</label>
              <div class="checkbox-group">
                <label v-for="tag in tags" :key="tag.id" class="form-checkbox">
                  <input v-model="postForm.tagIds" type="checkbox" :value="tag.id" />
                  <span>{{ tag.name }}</span>
                </label>
              </div>
            </div>

            <!-- Cover Image -->
            <div class="form-group">
              <label class="form-label">{{ t('admin.coverImage') }}</label>
              <div class="input-row">
                <input
                  v-model="postForm.coverImage"
                  type="text"
                  class="input"
                  placeholder="https://..."
                />
                <label class="btn btn-secondary">
                  <IconUpload />
                  <input
                    type="file"
                    accept="image/*"
                    class="sr-only"
                    :disabled="uploadingCover"
                    @change="uploadCoverImage"
                  />
                </label>
              </div>
              <div v-if="postForm.coverImage" class="mt-2">
                <AppImage :src="postForm.coverImage" alt="Preview" class="blog-admin-cover-preview" />
              </div>
            </div>

            <!-- Language tabs for translatable content -->
            <div class="form-divider">
              <span>{{ t('admin.content') }}</span>
            </div>

            <div class="tabs tabs--underline mb-4">
              <button
                v-for="locale in locales"
                :key="locale"
                type="button"
                class="tab"
                :class="{ 'is-active': activeLocaleTab === locale }"
                @click="activeLocaleTab = locale"
              >
                {{ getLocaleName(locale) }}
                <span
                  v-if="!postForm.translations[locale]?.title"
                  class="tab__indicator tab__indicator--warning"
                  :title="t('admin.missingTranslations')"
                ></span>
              </button>
            </div>

            <!-- Translation fields for active locale -->
            <div v-for="locale in locales" :key="locale" v-show="activeLocaleTab === locale">
              <div class="form-group">
                <label class="form-label">{{ t('admin.title') }}</label>
                <input
                  v-model="postForm.translations[locale].title"
                  type="text"
                  class="input"
                  @blur="generateSlug"
                />
              </div>

              <div class="form-group">
                <label class="form-label">{{ t('admin.excerpt') }}</label>
                <textarea
                  v-model="postForm.translations[locale].excerpt"
                  class="input"
                  rows="2"
                ></textarea>
              </div>

              <div class="form-group">
                <label class="form-label">{{ t('admin.content') }} (Markdown)</label>
                <textarea
                  v-model="postForm.translations[locale].content"
                  class="input input--code"
                  rows="12"
                ></textarea>
              </div>
            </div>
          </form>

          <footer class="modal-footer">
            <button type="button" class="btn btn-secondary" @click="showPostModal = false">
              {{ t('common.cancel') }}
            </button>
            <button
              type="submit"
              class="btn btn-primary"
              :disabled="saving"
              @click="savePost"
            >
              {{ saving ? t('common.saving') : t('common.save') }}
            </button>
          </footer>
        </div>
      </div>
    </Teleport>

    <!-- Category Modal with Language Tabs -->
    <Teleport to="body">
      <div v-if="showCategoryModal" class="modal-backdrop" @click.self="showCategoryModal = false">
        <div class="modal">
          <header class="modal-header">
            <h2>{{ editingCategory ? t('common.edit') : t('admin.addItem') }}</h2>
            <button class="btn btn-ghost btn-sm" @click="showCategoryModal = false">
              <IconX />
            </button>
          </header>

          <form class="modal-body" @submit.prevent="saveCategory">
            <div class="form-group">
              <label class="form-label" for="cat-slug">{{ t('admin.slug') }} *</label>
              <input
                id="cat-slug"
                v-model="categoryForm.slug"
                type="text"
                class="input"
                required
                pattern="[a-z0-9-]+"
              />
            </div>

            <!-- Language tabs -->
            <div class="form-divider">
              <span>{{ t('admin.content') }}</span>
            </div>

            <div class="tabs tabs--underline mb-4">
              <button
                v-for="locale in locales"
                :key="locale"
                type="button"
                class="tab"
                :class="{ 'is-active': activeCategoryLocaleTab === locale }"
                @click="activeCategoryLocaleTab = locale"
              >
                {{ getLocaleName(locale) }}
                <span
                  v-if="!categoryForm.translations[locale]?.name"
                  class="tab__indicator tab__indicator--warning"
                  :title="t('admin.missingTranslations')"
                ></span>
              </button>
            </div>

            <div v-for="locale in locales" :key="locale" v-show="activeCategoryLocaleTab === locale">
              <div class="form-group">
                <label class="form-label">{{ t('admin.name') }}</label>
                <input
                  v-model="categoryForm.translations[locale].name"
                  type="text"
                  class="input"
                  @blur="generateCategorySlug"
                />
              </div>
            </div>
          </form>

          <footer class="modal-footer">
            <button type="button" class="btn btn-secondary" @click="showCategoryModal = false">
              {{ t('common.cancel') }}
            </button>
            <button
              type="submit"
              class="btn btn-primary"
              :disabled="saving"
              @click="saveCategory"
            >
              {{ saving ? t('common.saving') : t('common.save') }}
            </button>
          </footer>
        </div>
      </div>
    </Teleport>

    <!-- Tag Modal (tags don't need translations in this system) -->
    <Teleport to="body">
      <div v-if="showTagModal" class="modal-backdrop" @click.self="showTagModal = false">
        <div class="modal">
          <header class="modal-header">
            <h2>{{ editingTag ? t('common.edit') : t('admin.addItem') }}</h2>
            <button class="btn btn-ghost btn-sm" @click="showTagModal = false">
              <IconX />
            </button>
          </header>

          <form class="modal-body" @submit.prevent="saveTag">
            <div class="form-group">
              <label class="form-label" for="tag-name">{{ t('admin.name') }} *</label>
              <input
                id="tag-name"
                v-model="tagForm.name"
                type="text"
                class="input"
                required
                @blur="generateTagSlug"
              />
            </div>

            <div class="form-group">
              <label class="form-label" for="tag-slug">{{ t('admin.slug') }} *</label>
              <input
                id="tag-slug"
                v-model="tagForm.slug"
                type="text"
                class="input"
                required
                pattern="[a-z0-9-]+"
              />
            </div>
          </form>

          <footer class="modal-footer">
            <button type="button" class="btn btn-secondary" @click="showTagModal = false">
              {{ t('common.cancel') }}
            </button>
            <button
              type="submit"
              class="btn btn-primary"
              :disabled="saving"
              @click="saveTag"
            >
              {{ saving ? t('common.saving') : t('common.save') }}
            </button>
          </footer>
        </div>
      </div>
    </Teleport>
  </div>
</template>
