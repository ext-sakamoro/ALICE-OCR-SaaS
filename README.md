# ALICE-OCR-SaaS

Optical Character Recognition SaaS built on the ALICE-OCR engine. Provides high-accuracy text extraction from images and PDFs with template matching, multi-language support, and batch processing via REST API.

## Architecture

```
Client --> API Gateway (8110) --> Core Engine (8113)
```

- **API Gateway**: Authentication, rate limiting, request proxying
- **Core Engine**: OCR inference, template management, batch scheduling

## Features

- Scene text and document OCR
- PDF and image (JPEG/PNG/TIFF/WebP) input
- Multi-language recognition (80+ scripts)
- Table and form structure extraction
- Template-based field extraction
- Batch processing with job tracking
- Bounding box and confidence output per character/word/line

## API Endpoints

### Core Engine (port 8113)

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Health check with uptime and stats |
| POST | `/api/v1/ocr/recognize` | Recognize text in a single image |
| POST | `/api/v1/ocr/batch` | Submit batch OCR job |
| GET | `/api/v1/ocr/templates` | List extraction templates |
| GET | `/api/v1/ocr/languages` | List supported OCR languages |
| GET | `/api/v1/ocr/stats` | Operational statistics |

### API Gateway (port 8110)

Proxies all `/api/v1/*` routes to the Core Engine with JWT/API-Key auth and token-bucket rate limiting.

## Quick Start

```bash
# Core Engine
cd services/core-engine
OCR_ADDR=0.0.0.0:8113 cargo run --release

# API Gateway
cd services/api-gateway
GATEWAY_ADDR=0.0.0.0:8110 CORE_ENGINE_URL=http://localhost:8113 cargo run --release
```

## Example Request

```bash
curl -X POST http://localhost:8113/api/v1/ocr/recognize \
  -H "Content-Type: application/json" \
  -d '{"image_b64":"...","language":"ja","detect_orientation":true}'
```

## License

AGPL-3.0-or-later. SaaS operators must publish complete service source code under AGPL-3.0.
