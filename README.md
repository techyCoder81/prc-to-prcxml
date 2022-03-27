# PRC to PRCXML

Welcome to the repository for **prc-to-prcxml**. This project aims to supply a plugin capable of taking existing `.prc` (and potentially other) file formats in your `/ultimate/mods/` folder on SD and translating them at runtime into the `.prcxml` format, during ultimate startup. The generated output is compatible with `arcropolis` and contains only values which differ from vanilla. The purpose of this is to make translating existing mods into xml significantly easier, because working with PRCs in xml format is **significantly** easier than working with prcs in the original format, and xml format is also diffable by git.


# How to use this

First get the plugin from the [releases page](https://github.com/techyCoder81/prc-to-prcxml/releases/), and place it in `/atmosphere/contents/<titleid>/romfs/skyline/plugins/`. Then, start smash.

If you have `cargo-skyline` installed already, you can run `cargo skyline listen` to observe the output for the duration of operation. Without this, there is no meaningful way in real time to observe progress beyond some popups with info after each step, or it (maybe) crashing. Additionally, a log file is populated during runtime in `sd:/prc_to_prcxml.log`, though be aware that this file may be relatively large, depending on your mod setup. If you wish to report a bug or broken functionality, please attach this file to any issues created here.

During startup, just before you reach the title screen, a prompt may ask you if you want to delete the `/xml/` directory - this is where prc-to-prcxml stores the generated prcxml output. Pick an option. The tool will then begin walking your mod directories to find any `.prc` files. In the meantime, ult will keep loading. You should also see prc_to_prcxml.nro loaded! in the version string section of your splash screen. **Please be aware, the plugin will be triggered a second time if you cause the game to reload the splash screen, such as if the "how to play" video plays, and the game returns to the splash screen.** This is very likely not something you will want, so please do not idle on the splash screen.

After a moment (the plugin is running in the background), a popup will ask you if you also want to delete the original `.prc` files. Select yes or no.

Then, it will ask you if you want to move the newly generated xml files into your original mods directories automatically. Select yes or no.

At this point, the plugin will inform you that conversion is complete, and report any files that could not be translated. Any files that could not be translated will not be affected by this operation, and will remain in your `/ultimate/mods/` directories as unchanged `.prc` files. One example of such an issue is HDR's `ui_series_db.prc` file.

# Outputs
1. `sd:/xml/` will contain all of the translated prcxml files, which should be compatible with arcropolis as-is.
2. If you so chose, the original `.prc` files may have been deleted from each respective mod folder in `sd:/ultimate/mods/`.
3. If you so chose, the new `.prcxml` files may have been copied into `sd:/ultimate/mods` into each original mod folder.
4. `sd:/prc_to_prcxml.log` will contain the logging output from the plugin operations.


# Contributing
This tool is entirely open source and is open to pull requests for potential fixes or improvements. Feel free to make issues for any legitimate bugs or problems you may find, but please make sure to attach the logs.
