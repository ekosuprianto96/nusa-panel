import { authService } from './auth.service'
import { fileService } from './file.service'
import { userService } from './user.service'
import { systemService } from './system.service'
import { domainService } from './domain.service'
import { databaseService } from './database.service'
import { webserverService } from './webserver.service'
import { securityService } from './security.service'
import { appService } from './app.service'
import { redisService } from './redis.service'
import { ftpService } from './ftp.service'
import { emailService } from './email.service'

/**
 * Barrel file untuk mengekspor semua service dari satu tempat.
 * Memudahkan penggunaan di komponen Vue.
 */
export {
  authService,
  fileService,
  userService,
  systemService,
  domainService,
  databaseService,
  webserverService,
  securityService,
  appService,
  redisService,
  ftpService,
  emailService
}

// Default export as a unified object if needed
export default {
  auth: authService,
  files: fileService,
  users: userService,
  system: systemService,
  domains: domainService,
  databases: databaseService,
  webserver: webserverService,
  security: securityService,
  apps: appService,
  redis: redisService,
  ftp: ftpService,
  emails: emailService
}
