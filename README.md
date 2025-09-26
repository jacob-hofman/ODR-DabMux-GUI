## A control UI for ODR-DabMux

Goals of this Web User Interface:

 * Simplify creating basic ODR-DabMux DAB Ensemble configurations
 * Interact with the ODR-DabMux Remote Control through a web UI

### Build
 * Install Rust, most probably through [rustup](https://rustup.rs/)
 * Compile `odr-dabmux-gui` :
   ```
   cargo build --release
   ```

### Installation
The compiled executable is self-contained and does not require any additional file from the project directory. You can copy it to any directory in your path, for example:
  ```
  sudo cp target/release/odr-dabmux-gui /usr/local/bin
  ```

### Usage
 * Run `odr-dabmux-gui` :
   ```
   odr-dabmux-gui --port 3000
   ```
 * Navigate to http://localhost:3000
 * Create a new Ensemble configuration in the Settings page, and specify where to write the odr-dabmux json config file
 * Execute `odr-dabmux` with one argument: the configuration file
 * Check in the Dashboard page that you see RC values
