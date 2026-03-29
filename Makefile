.PHONY: install build lint test dev clean typecheck tauri

install:
	pnpm install

build:
	pnpm run build

lint:
	pnpm run lint

test:
	pnpm run test

typecheck:
	pnpm run typecheck

dev:
	pnpm run dev

tauri:
	pnpm run tauri dev

clean:
	rm -rf dist coverage node_modules src-tauri/target
