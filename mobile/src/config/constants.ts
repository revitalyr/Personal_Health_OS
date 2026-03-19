export const API_BASE_URL = __DEV__ 
  ? 'http://localhost:8080' 
  : 'https://api.healthos.app';

export const GOOGLE_WEB_CLIENT_ID = 'YOUR_GOOGLE_WEB_CLIENT_ID';
export const GOOGLE_IOS_CLIENT_ID = 'YOUR_GOOGLE_IOS_CLIENT_ID';
export const GOOGLE_ANDROID_CLIENT_ID = 'YOUR_GOOGLE_ANDROID_CLIENT_ID';

export const MAX_FILE_SIZE = 10 * 1024 * 1024; // 10MB
export const SUPPORTED_DOCUMENT_TYPES = [
  'image/jpeg',
  'image/png',
  'application/pdf',
  'application/dicom',
];

export const MEDICATION_REMINDER_TIMES = [
  '08:00',
  '12:00',
  '18:00',
  '22:00',
];
