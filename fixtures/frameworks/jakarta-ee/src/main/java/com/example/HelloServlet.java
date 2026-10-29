package com.example;

import jakarta.servlet.http.HttpServlet;
import jakarta.servlet.annotation.WebServlet;
import jakarta.annotation.PostConstruct;

@WebServlet(urlPatterns = "/hello")
public class HelloServlet extends HttpServlet {

    @PostConstruct
    public void init() {
        // initialization
    }

    @Override
    protected void doGet(jakarta.servlet.http.HttpServletRequest req,
                         jakarta.servlet.http.HttpServletResponse resp) {
        resp.getWriter().write("Hello Jakarta EE");
    }
}
