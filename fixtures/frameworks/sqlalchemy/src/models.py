from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker, DeclarativeBase

class Base(DeclarativeBase):
    pass

engine = create_engine("sqlite:///:memory:")
Session = sessionmaker(bind=engine)
