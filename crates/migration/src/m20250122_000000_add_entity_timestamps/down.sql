-- Drop indexes
DROP INDEX IF EXISTS "idx_song_created_at";
DROP INDEX IF EXISTS "idx_release_created_at"; 
DROP INDEX IF EXISTS "idx_artist_created_at";
DROP INDEX IF EXISTS "idx_event_created_at";
DROP INDEX IF EXISTS "idx_tag_created_at";

-- Drop triggers
DROP TRIGGER IF EXISTS update_song_updated_at ON "public"."song";
DROP TRIGGER IF EXISTS update_release_updated_at ON "public"."release";
DROP TRIGGER IF EXISTS update_artist_updated_at ON "public"."artist";
DROP TRIGGER IF EXISTS update_event_updated_at ON "public"."event";
DROP TRIGGER IF EXISTS update_tag_updated_at ON "public"."tag";

-- Remove timestamp columns
ALTER TABLE "public"."song"
DROP COLUMN IF EXISTS "created_at",
DROP COLUMN IF EXISTS "updated_at";

ALTER TABLE "public"."release"
DROP COLUMN IF EXISTS "created_at",
DROP COLUMN IF EXISTS "updated_at";

ALTER TABLE "public"."artist"
DROP COLUMN IF EXISTS "created_at",
DROP COLUMN IF EXISTS "updated_at";

ALTER TABLE "public"."event"
DROP COLUMN IF EXISTS "created_at",
DROP COLUMN IF EXISTS "updated_at";

ALTER TABLE "public"."tag"
DROP COLUMN IF EXISTS "created_at",
DROP COLUMN IF EXISTS "updated_at";