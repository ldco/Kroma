CREATE TABLE `audit_logs` (
	`id` integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	`action` text NOT NULL,
	`user_id` integer,
	`target_user_id` integer,
	`ip_address` text,
	`user_agent` text,
	`details` text,
	`success` integer DEFAULT true,
	`created_at` integer,
	FOREIGN KEY (`user_id`) REFERENCES `users`(`id`) ON UPDATE no action ON DELETE set null,
	FOREIGN KEY (`target_user_id`) REFERENCES `users`(`id`) ON UPDATE no action ON DELETE set null
);
--> statement-breakpoint
CREATE INDEX `audit_logs_created_idx` ON `audit_logs` (`created_at`);--> statement-breakpoint
CREATE INDEX `audit_logs_action_idx` ON `audit_logs` (`action`);--> statement-breakpoint
CREATE TABLE `portfolios` (
	`id` integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	`slug` text NOT NULL,
	`name` text NOT NULL,
	`description` text,
	`type` text NOT NULL,
	`cover_image_url` text,
	`cover_thumbnail_url` text,
	`order` integer DEFAULT 0,
	`published` integer DEFAULT false,
	`created_at` integer,
	`updated_at` integer
);
--> statement-breakpoint
CREATE UNIQUE INDEX `portfolios_slug_unique` ON `portfolios` (`slug`);--> statement-breakpoint
CREATE INDEX `portfolios_published_idx` ON `portfolios` (`published`);--> statement-breakpoint
CREATE TABLE `pricing_features` (
	`id` integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	`tier_id` integer NOT NULL,
	`text` text NOT NULL,
	`included` integer DEFAULT true,
	`order` integer DEFAULT 0,
	`created_at` integer,
	FOREIGN KEY (`tier_id`) REFERENCES `pricing_tiers`(`id`) ON UPDATE no action ON DELETE cascade
);
--> statement-breakpoint
CREATE INDEX `pricing_features_tier_idx` ON `pricing_features` (`tier_id`);--> statement-breakpoint
CREATE TABLE `pricing_tiers` (
	`id` integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	`slug` text NOT NULL,
	`name` text NOT NULL,
	`description` text,
	`price` integer,
	`currency` text DEFAULT 'USD',
	`period` text DEFAULT 'month',
	`featured` integer DEFAULT false,
	`cta_text` text,
	`cta_url` text DEFAULT '/contact',
	`order` integer DEFAULT 0,
	`published` integer DEFAULT true,
	`created_at` integer,
	`updated_at` integer
);
--> statement-breakpoint
CREATE UNIQUE INDEX `pricing_tiers_slug_unique` ON `pricing_tiers` (`slug`);--> statement-breakpoint
PRAGMA foreign_keys=OFF;--> statement-breakpoint
CREATE TABLE `__new_portfolio_items` (
	`id` integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	`portfolio_id` integer NOT NULL,
	`item_type` text NOT NULL,
	`order` integer DEFAULT 0,
	`published` integer DEFAULT true,
	`created_at` integer,
	`updated_at` integer,
	`media_url` text,
	`thumbnail_url` text,
	`caption` text,
	`slug` text,
	`title` text,
	`description` text,
	`content` text,
	`tags` text,
	`category` text,
	`published_at` integer,
	FOREIGN KEY (`portfolio_id`) REFERENCES `portfolios`(`id`) ON UPDATE no action ON DELETE cascade
);
--> statement-breakpoint
INSERT INTO `__new_portfolio_items`("id", "portfolio_id", "item_type", "order", "published", "created_at", "updated_at", "media_url", "thumbnail_url", "caption", "slug", "title", "description", "content", "tags", "category", "published_at") SELECT "id", "portfolio_id", "item_type", "order", "published", "created_at", "updated_at", "media_url", "thumbnail_url", "caption", "slug", "title", "description", "content", "tags", "category", "published_at" FROM `portfolio_items`;--> statement-breakpoint
DROP TABLE `portfolio_items`;--> statement-breakpoint
ALTER TABLE `__new_portfolio_items` RENAME TO `portfolio_items`;--> statement-breakpoint
PRAGMA foreign_keys=ON;--> statement-breakpoint
CREATE INDEX `portfolio_items_portfolio_idx` ON `portfolio_items` (`portfolio_id`);--> statement-breakpoint
CREATE TABLE `__new_users` (
	`id` integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	`email` text NOT NULL,
	`password_hash` text NOT NULL,
	`name` text,
	`role` text DEFAULT 'editor' NOT NULL,
	`failed_login_attempts` integer DEFAULT 0,
	`locked_until` integer,
	`last_failed_login` integer,
	`created_at` integer,
	`updated_at` integer
);
--> statement-breakpoint
INSERT INTO `__new_users`("id", "email", "password_hash", "name", "role", "failed_login_attempts", "locked_until", "last_failed_login", "created_at", "updated_at") SELECT "id", "email", "password_hash", "name", "role", "failed_login_attempts", "locked_until", "last_failed_login", "created_at", "updated_at" FROM `users`;--> statement-breakpoint
DROP TABLE `users`;--> statement-breakpoint
ALTER TABLE `__new_users` RENAME TO `users`;--> statement-breakpoint
CREATE UNIQUE INDEX `users_email_unique` ON `users` (`email`);--> statement-breakpoint
CREATE INDEX `sessions_user_expires_idx` ON `sessions` (`user_id`,`expires_at`);--> statement-breakpoint
CREATE INDEX `sessions_expires_idx` ON `sessions` (`expires_at`);--> statement-breakpoint
CREATE UNIQUE INDEX `translations_locale_key_unique` ON `translations` (`locale`,`key`);