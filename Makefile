.PHONY: dev app dmg install test clean

install:
	npm install

# Launch the Tauri app in development mode (hot reload)
dev: install
	npm run tauri -- dev

# Build the macOS .app bundle -> src-tauri/target/release/bundle/macos/
app: install
	npm run tauri -- build --bundles app

# Build the macOS .dmg installer -> src-tauri/target/release/bundle/dmg/
dmg: install
	npm run tauri -- build --bundles dmg

test:
	npm test -- --run
	cd src-tauri && cargo test

clean:
	rm -rf dist src-tauri/target
