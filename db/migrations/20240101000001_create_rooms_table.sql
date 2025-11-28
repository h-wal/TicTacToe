CREATE TABLE IF NOT EXISTS rooms (
	id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	slug VARCHAR(255) NOT NULL,
	creator_id VARCHAR(255) NOT NULL REFERENCES users(username),
	status VARCHAR(50) NOT NULL DEFAULT 'waiting',
	winner VARCHAR(255) REFERENCES users(username),
	created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
	updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_rooms_roomslug ON rooms(slug);