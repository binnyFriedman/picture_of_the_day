# Bing Picture of the day

A simple cli program that downloads the Bing picture of the day and saves it to your computer.
It also saves yesterday's picture to a folder called "archive".

Run build command to compile the program.
```bash
$ cargo build --release  
```
Adding a cron job to run the program every day at midnight is recommended.
On macOS, you can do this with adding a cron job to your crontab.
```shell
$ 0 00 * * *  /path/to/program/target/release/picture_of_the_day
```
This is only for Linux,macOS. since windows has it built in.

Then set the wallpaper's directory to the picture of the day.<br>
[MacOS](https://support.apple.com/en-au/HT207703)

Enjoy!

