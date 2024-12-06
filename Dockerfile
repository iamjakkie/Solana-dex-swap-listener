FROM node:18-alpine

WORKDIR /app

COPY package.json ./
RUN npm install --production

COPY raydium.js ./
COPY .env ./

EXPOSE 3000

CMD ["node", "raydium.js"]