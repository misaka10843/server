-- Add created_at and updated_at columns to main entity tables
-- Song table
ALTER TABLE "public"."song"
ADD COLUMN "created_at" timestamptz NOT NULL DEFAULT NOW(),
ADD COLUMN "updated_at" timestamptz NOT NULL DEFAULT NOW();

-- Release table  
ALTER TABLE "public"."release"
ADD COLUMN "created_at" timestamptz NOT NULL DEFAULT NOW(),
ADD COLUMN "updated_at" timestamptz NOT NULL DEFAULT NOW();

-- Artist table
ALTER TABLE "public"."artist"
ADD COLUMN "created_at" timestamptz NOT NULL DEFAULT NOW(),
ADD COLUMN "updated_at" timestamptz NOT NULL DEFAULT NOW();

-- Event table
ALTER TABLE "public"."event"
ADD COLUMN "created_at" timestamptz NOT NULL DEFAULT NOW(),
ADD COLUMN "updated_at" timestamptz NOT NULL DEFAULT NOW();

-- Tag table
ALTER TABLE "public"."tag"
ADD COLUMN "created_at" timestamptz NOT NULL DEFAULT NOW(),
ADD COLUMN "updated_at" timestamptz NOT NULL DEFAULT NOW();

-- Create triggers for automatic updated_at updates
CREATE TRIGGER update_song_updated_at BEFORE UPDATE
ON "public"."song" FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_release_updated_at BEFORE UPDATE
ON "public"."release" FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_artist_updated_at BEFORE UPDATE
ON "public"."artist" FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_event_updated_at BEFORE UPDATE
ON "public"."event" FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_tag_updated_at BEFORE UPDATE
ON "public"."tag" FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- Create indexes for efficient ordering by created_at
CREATE INDEX "idx_song_created_at" ON "public"."song" ("created_at" DESC);
CREATE INDEX "idx_release_created_at" ON "public"."release" ("created_at" DESC);
CREATE INDEX "idx_artist_created_at" ON "public"."artist" ("created_at" DESC);
CREATE INDEX "idx_event_created_at" ON "public"."event" ("created_at" DESC);
CREATE INDEX "idx_tag_created_at" ON "public"."tag" ("created_at" DESC);