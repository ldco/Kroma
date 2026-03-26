# Admin Panel User Guide

A guide for content managers and administrators using the Puppet Master admin panel.

## Accessing the Admin Panel

### Login

Navigate to the admin login page:
- **Website-admin mode** (default): `/admin/login`
- **Website-app mode**: `/login` (visible in header)
- **App-only mode**: `/login` (root redirects here)

Enter your email and password. Check "Remember me" to stay logged in for 7 days.

### Dashboard

After login, you'll see the dashboard with:
- Quick statistics (portfolios, contacts, users)
- Unread contact messages count
- Navigation to all admin sections

---

## Site Settings

Manage your website's contact information, social links, SEO, and more.

### Accessing Settings

Click **Settings** in the admin navigation.

### Setting Groups

Settings are organized into groups:

| Group | Description |
|-------|-------------|
| Contact | Email, phone, address |
| Social | Social media links |
| Legal | Company info, tax IDs |
| Footer | CTA button, legal links |
| SEO | Page title, description, keywords |
| Analytics | Google Analytics, Yandex Metrica |
| Verification | Search console verification codes |

### Editing Settings

1. Find the setting you want to change
2. Enter the new value in the input field
3. Click **Save** (changes are saved automatically for some fields)

### Tips

- **Leave empty** to hide: If a setting is empty, related UI elements won't appear
- **Social links**: Enter full URLs (e.g., `https://instagram.com/yourpage`)
- **Phone numbers**: Use international format (e.g., `+1234567890`)

---

## Portfolio Management

Create and manage portfolio galleries and case studies.

### Portfolio List

Click **Portfolio** in the admin navigation to see all portfolios.

### Creating a Portfolio

1. Click **New Portfolio**
2. Fill in the details:
   - **Name**: Display name for the portfolio
   - **Slug**: URL-friendly identifier (auto-generated from name)
   - **Description**: Brief description
   - **Type**: Gallery (images/videos) or Case Study
3. Click **Create**

### Portfolio Types

| Type | Description | Best For |
|------|-------------|----------|
| Gallery | Grid of images/videos | Photo portfolios, project screenshots |
| Case Study | Detailed project writeups | In-depth project descriptions |

### Managing Portfolio Items

#### Adding Images

1. Open a portfolio
2. Click **Add Item** or drag files to the upload area
3. Images are automatically:
   - Converted to WebP format
   - Resized to max dimensions
   - Thumbnails generated

#### Adding Videos

1. Click **Add Video**
2. Select a video file (MP4, WebM, MOV, AVI)
3. Videos are processed and thumbnails created

#### Item Details

For each item you can set:
- **Caption**: Text shown below the item
- **Order**: Drag to reorder items

#### Gallery Item Actions

- **View**: Preview the full-size image/video
- **Edit**: Change caption
- **Delete**: Remove from portfolio

### Publishing

- **Draft**: Portfolio is hidden from public
- **Published**: Portfolio appears on the website

Toggle the publish status using the switch next to the portfolio name.

---

## Translations

Manage all text content across different languages.

### Accessing Translations

Click **Translations** in the admin navigation.

### Translation Types

| Type | Description | Who Can Edit |
|------|-------------|--------------|
| Content | Headings, paragraphs, CTA text | All editors |
| System | Navigation, buttons, labels | Master only |

### Editing Translations

1. Select the language tab (EN, RU, HE, etc.)
2. Find the translation key you want to edit
3. Enter the new text
4. Click **Save**

### Translation Keys

Keys use dot notation to organize content:

```
hero.title        → Homepage hero section title
hero.subtitle     → Homepage hero section subtitle
nav.home          → "Home" navigation link
cta.contact       → "Contact Us" button text
footer.copyright  → Footer copyright text
```

### Adding New Translations

1. Click **Add Translation**
2. Enter the key (e.g., `section.newKey`)
3. Enter the value for the current language
4. Switch to other languages and add translations

### Best Practices

- Keep translation keys consistent across languages
- Use meaningful keys that describe the content location
- Test translations on the live site to check formatting

---

## Contact Messages

View and manage contact form submissions.

### Accessing Contacts

Click **Contacts** in the admin navigation.

### Message List

Messages are displayed with:
- Sender name and email
- Message preview
- Date received
- Read/unread status

### Viewing Messages

Click a message to see the full content.

### Actions

| Action | Description |
|--------|-------------|
| Mark as Read | Remove from unread count |
| Mark as Unread | Add back to unread count |
| Delete | Permanently remove message |

### Notifications

If configured, you'll receive notifications for new messages via:
- Email (to the admin email)
- Telegram (to the configured chat)

---

## User Management

Manage admin panel users (Admin and Master roles only).

### Accessing Users

Click **Users** in the admin navigation.

### User List

Shows all users with:
- Name and email
- Role
- Creation date
- Account status (active/locked)

### Creating Users

1. Click **New User**
2. Fill in:
   - **Email**: Login email (must be unique)
   - **Name**: Display name
   - **Password**: Temporary password
   - **Role**: Editor, Admin, or Master
3. Click **Create**

### Roles

| Role | Access Level |
|------|--------------|
| Editor | Settings, portfolios, translations, contacts |
| Admin | All above + user management |
| Master | All above + system health, audit logs |

### Editing Users

1. Click a user row
2. Update details
3. Click **Save**

You can change:
- Name
- Email
- Role (admins cannot change master accounts)

### Resetting Passwords

1. Click **Reset Password** on a user
2. Enter new password
3. User will need to login with new password

### Account Lockout

If a user has too many failed login attempts, their account is locked.

To unlock:
1. Find the locked user (shown with lock icon)
2. Click **Unlock Account**

---

## System Health

Monitor system status (Master role only).

### Accessing Health

Click **Health** in the admin navigation.

### Health Checks

| Check | What It Shows |
|-------|---------------|
| Database | Connection status, query latency |
| Memory | Current usage vs available |
| Uptime | Server uptime duration |
| Version | Application version |

### Status Indicators

| Status | Meaning |
|--------|---------|
| OK (green) | Everything working normally |
| Degraded (yellow) | Working but with issues |
| Unhealthy (red) | Critical problem |

### Application Logs

View recent application logs for troubleshooting:
- Error messages
- Warning messages
- Info messages

### Audit Logs

Security event log showing:
- Login attempts (successful and failed)
- User changes
- Account lockouts

---

## Common Tasks

### Change Your Password

1. Click your name in the header
2. Select **Change Password**
3. Enter current password
4. Enter and confirm new password
5. Click **Update**

### Logout

Click the logout icon in the header or select **Logout** from the user menu.

### Switch Language

If multiple languages are enabled, use the language switcher in the header.

### Switch Theme

Click the sun/moon icon to toggle between light and dark mode.

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Esc` | Close modal/dialog |
| `Enter` | Submit form (when focused) |

---

## Troubleshooting

### Can't Login

1. **Check credentials**: Ensure email and password are correct
2. **Account locked**: Wait 30 minutes or contact an admin
3. **Clear cookies**: Try clearing browser cookies and cache

### Changes Not Saving

1. **Check connection**: Ensure you're online
2. **Refresh page**: Try refreshing and making changes again
3. **Check permissions**: Some changes require higher roles

### Images Not Uploading

1. **Check file size**: Max 10MB for images
2. **Check format**: Use JPEG, PNG, WebP, or GIF
3. **Check connection**: Large files need stable connection

### Contact Messages Not Arriving

1. **Check notification settings**: Ensure email/Telegram is configured
2. **Check spam folder**: Notifications might be filtered
3. **Test contact form**: Submit a test message

---

## Getting Help

If you encounter issues:

1. Check this guide for answers
2. Contact your site administrator
3. Developers can check the technical documentation
