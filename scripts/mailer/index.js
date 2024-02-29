#!/usr/bin/env node

const fs = require('fs');
const yaml = require('js-yaml');
const nodemailer = require('nodemailer');

let config = undefined;

try {
    let configfile = fs.readFileSync('../../.env', 'utf8');

    config = yaml.load(configfile);
} catch (e) {
    console.log("Config Load failed: " + e);
}

if(config) {
  var transporter = nodemailer.createTransport({
    name: config.component + '.local',
    host: config.smtp.host,
    port: config.smtp.port,
    secure: false,
    requireTLS: true,
    auth: {
      user: config.smtp.login,
      pass: config.smtp.password
    },
    logger: true,
    debug: true,
  });

  var mailOptions = {
    from: config.smtp.email_address,
    to: config.smtp.email_address,
    subject: '[Mail Test - ' + config.component + '] Node Email Test ('
      + process.env.GITHUB_RUN_ID + ')',
    text: 'Mail Test (' + process.env.GITHUB_RUN_ID + ') - ' + config.component
      + "\n=============================\n\nmy Node test email message"
  };

  transporter.sendMail(mailOptions, function(error, info){
    if (error) {
      console.log("Email failed: " + error);
    } else {
      console.log('Email sent: ' + info.response);
    }
  });
}