# For now create the basis file outside of rafx/distill as it requires reading data from multiple files for a single
# asset, and this isn't supported yet.

basisu -mipmap -uastc -tex_type cubemap right.png left.png front.png back.png top.png bot.png