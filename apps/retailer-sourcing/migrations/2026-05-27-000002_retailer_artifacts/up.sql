CREATE TABLE retailer_sitemap_files (
    id UUID PRIMARY KEY,
    retailer_code TEXT NOT NULL,
    url TEXT NOT NULL,
    content TEXT NOT NULL,
    status_code INT NOT NULL,
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_retailer_sitemap_files_retailer_code ON retailer_sitemap_files (retailer_code);

CREATE TABLE retailer_homepages (
    id UUID PRIMARY KEY,
    retailer_code TEXT NOT NULL,
    url TEXT NOT NULL,
    content TEXT NOT NULL,
    status_code INT NOT NULL,
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_retailer_homepages_retailer_code ON retailer_homepages (retailer_code);
