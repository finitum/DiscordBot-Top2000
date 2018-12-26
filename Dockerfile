FROM python:3.6-alpine

# Requirements
RUN apk add --no-cache ffmpeg build-base libffi-dev opus-dev
RUN pip install pipenv

RUN mkdir -p /app
COPY api.py /app/
COPY bot.py /app/
COPY Pipfile /app/
COPY Pipfile.lock /app/
WORKDIR /app
RUN pipenv install --deploy --system

CMD ["python", "-u", "bot.py"]
