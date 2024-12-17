## A control UI for ODR-DabMux

Goals of this Web User Interface:

 * Simplify creating basic ODR-DabMux DAB Ensemble configurations
 * Interact with the ODR-DabMux Remote Control through a web UI

Complilation prerequisites

 * Install Rust, most probably through [rustup](https://rustup.rs/)
 * Compile and install `odr-dabmux-gui` :
   ```
   sudo -E $HOME/.cargo/bin/cargo install --root /usr/local --path .
   ```
 * Run `odr-dabmux-gui` :
   ```
   odr-dabmux-gui --port 3000
   ```
 * Navigate to http://localhost:3000
 * Create a new Ensemble configuration in the Settings page, and specify where to write the odr-dabmux json config file
 * Execute `odr-dabmux` with one argument: the configuration file
 * Check in the Dashboard page that you see RC values
