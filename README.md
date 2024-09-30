# Playing content with HyperDeck player remote control

![screenshot](/screenshot.png?raw=true)

## Prerequisites

Please ensure that all the clips have [the same format, frame rate, 
and resolution](https://forum.blackmagicdesign.com/viewtopic.php?p=670924#p670924), 
otherwise the HyperDeck will not play all the clips, only some of them.
You can, however, create separate playlists/timelines for parts of your 
screening, where the individual playlists contain only clips with the 
same frame rate (converting media to the same format and resolution 
should be doable; you shouldn't, of course, change the framerate of 
an invidual clip).

Connect your HyperDeck with the laptop using an ethernet cable.

## Connecting

Open the HyperDeck player remote control application and enter the 
IP address the HyperDeck got assigned. To get this IP address, push 
the MENU button on the HyperDeck, scroll to Setup, press SET, 
and scroll down until 'IP Address' is visible in the display. Click 
the Connect button in the application. In the bottom part of the 
window you will see a message from your HyperDeck containing the 
protocol version and the model of the device.

## Reading the disk list and creating a playlist

First step is to press 'Clear timeline' to clear the initial timeline 
in the HyperDeck (which may or may not be an alphabetically sorted 
list of files contained on the storage medium) and retrieve a list of 
file names, lengths, etc. This will appear on the left side of the 
screen. Clicking on a file puts the selected clip in the timeline, 
which appears on the right.

## Playing the timeline

When you are finished, press the Play button in the application. 
During playback, all controls except the Pause button will be 
disabled for safety reasons. Do NOT press play on the HyperDeck, as 
the application simply doesn't get a notification that playback has 
started, interfering with the user interface (e.g., controls are not 
locked and changing the timeline or, worse, unintentionally pressing 
'Clear timeline' will result in unwanted behaviour).

## Saving and opening the timeline

After you created a timeline, press the Save button to save the 
current timeline. If you want to load a previously saved timeline, 
just click the Open button and select a file. Please note that after 
connecting the Hyperdeck, one should always press 'Clear timeline' 
before opening a timeline file, otherwise you might get messages 
about clips not being found and your playlist will not be correct.

Please bear in mind that once you pull the power chord of the 
HyperDeck, your timeline will be erased. After a power-up the 
HyperDeck will reset the timeline to its initial state 
(i.e., the above mentioned possibly alphabetically sorted list).

# Disclaimer

This little piece of software is made with love (for projecting 
movies), but may or may not contain bugs and other critical errors. 
The code is not modularized, everything runs in one Svelte page, and 
the Rust backend can surely be improved. In short: do not take this 
code as a great example of a Tauri application, nor be surprised if 
this pesky little thing will make dinosaurs alive, again.

HyperDeck is a registered trademark of Blackmagic Design Pty. Ltd.

## Open-source software

This HyperDeck player remote control makes use of the following 
pieces of open-source software components:

- [Tauri](https://tauri.app), MIT license
- [Serde](https://crates.io/crates/serde), MIT or Apache License v2.0
- [Serde JSON](https://crates.io/crates/serde_json), MIT or Apache License v2.0
- [telnet-rs](https://crates.io/crates/telnet), MIT license
- [Tokio](https://crates.io/crates/tokio), MIT license
- [Svelte](https://svelte.dev), MIT license
- [Material Symbols](https://fonts.google.com/icons), Apache License v2.0
