import { Controller, Get, Module } from "@nestjs/common";

@Controller("users")
export class UsersController {
  @Get()
  findAll() {
    return { users: [] };
  }
}

@Module({
  controllers: [UsersController],
})
export class AppModule {}
