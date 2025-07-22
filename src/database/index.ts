import { PrismaClient } from "@prisma/client";
import { GuildMember } from "discord.js";

export const prisma = new PrismaClient()

export async function getUser(member: GuildMember) {

    const data = await prisma;

}