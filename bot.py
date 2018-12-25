import discord
import time
import api
from discord.ext import commands
from datetime import datetime

# Creates the bot and specifies the prefix
bot = commands.Bot(command_prefix="!")
players = {}
channels = []


@bot.event
async def on_ready():
    for channel in bot.get_all_channels():
        if channel.name == "TOP2000":
            await bot.join_voice_channel(channel)
    print("Started...")


@bot.command(pass_context=True)
async def play(ctx):
    server = ctx.message.server
    voice_client = bot.voice_client_in(server)
    player = await voice_client.create_ytdl_player("http://icecast.omroep.nl/radio2-bb-mp3")
    player.start()
    players[server.id] = player
    channels.append(ctx.message.channel)
    print("Playing")
    await schedule_info()


@bot.command(pass_context=True)
async def pause(ctx):
    server = ctx.message.server
    players[server.id].stop()


def generate_current_song_embed():
    data = api.get_now_on_air()["songfile"]

    embed = discord.Embed(
        title=data["artist"],
        description=data["title"]
    )

    img = api.get_img_url(data['songversion']['image'][0]['url'])

    embed.set_thumbnail(url=img)

    return embed


@bot.command()
async def song():
    await bot.say(embed=generate_current_song_embed())


async def schedule_info():
    for c in channels:
        await bot.send_message(c, embed=generate_current_song_embed())

    end = datetime.strptime(api.get_now_on_air()['stopdatetime'], "%Y-%m-%dT%H:%M:%S+01:00")
    now = datetime.now()
    run_at = end - now
    delay = run_at.total_seconds()
    time.sleep(delay)
    await schedule_info()


if __name__ == "__main__":
    bot.run("NTI3MTE1MTk1MDU5OTI5MTE4.DwPPPA.9yeoo_IGMzMVAwBLfATt6LdVCaw")  # Needs to be moved to a file
