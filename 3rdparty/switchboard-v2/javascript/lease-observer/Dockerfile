FROM node:14
WORKDIR /
COPY package*.json ./
RUN npm install
COPY . .
CMD [ "npm", "run", "start" ]
