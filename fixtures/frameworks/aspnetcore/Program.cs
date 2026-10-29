using Microsoft.AspNetCore.Mvc;
using Microsoft.AspNetCore.Builder;
using Microsoft.Extensions.DependencyInjection;

var builder = WebApplication.CreateBuilder(args);
builder.Services.AddControllers();

var app = builder.Build();
app.MapControllers();
app.Run();

[ApiController]
[Route("[controller]")]
public class WeatherController : ControllerBase
{
    [HttpGet]
    public string Get() => "Hello ASP.NET Core";
}
