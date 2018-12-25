import discord
import api
from discord.ext import commands

# Creates the bot and specifies the prefix
bot = commands.Bot(command_prefix="!")
players = {}


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
    print("Playing")


@bot.command(pass_context=True)
async def pause(ctx):
    server = ctx.message.server
    players[server.id].stop()


@bot.command(pass_context=True)
async def song(ctx):
    data = api.get_now_on_air()["songfile"]

    embed = discord.Embed(
        title=data["artist"],
        description=data["title"]
    )

    img = api.get_img_url(data['songversion']['image'][0]['url'])

    embed.set_thumbnail(url=img)

    await bot.say(embed=embed)

# actually run the bot
bot.run("NTI3MTE1MTk1MDU5OTI5MTE4.DwPPPA.9yeoo_IGMzMVAwBLfATt6LdVCaw")  # Needs to be moved to a file
