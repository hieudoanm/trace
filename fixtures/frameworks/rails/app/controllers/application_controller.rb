class ApplicationController < ActionController::Base
end

class WelcomeController < ApplicationController
  def index
    render plain: "Hello Rails"
  end
end
