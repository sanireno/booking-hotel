// Set this once before loading app.js when deploying the frontend elsewhere.
export const API_BASE_URL = window.BOOKING_API_URL || 'http://127.0.0.1:3000';

export class ApiError extends Error {
  constructor(message, status = 0) { super(message); this.status = status; }
}

export async function request(path, { method = 'GET', body, token, signal } = {}) {
  const headers = { Accept: 'application/json' };
  if (body !== undefined) headers['Content-Type'] = 'application/json';
  if (token) headers.Authorization = `Bearer ${token}`;
  let response;
  try {
    response = await fetch(`${API_BASE_URL}${path}`, {
      method, headers, body: body === undefined ? undefined : JSON.stringify(body),
      signal: signal ? AbortSignal.any([signal, AbortSignal.timeout(15000)]) : AbortSignal.timeout(15000),
      credentials: 'omit', cache: 'no-store',
    });
  } catch (error) {
    if (error.name === 'AbortError') throw error;
    throw new ApiError('Не удалось связаться с сервером. Попробуйте еще раз.');
  }
  const data = response.status === 204 ? null : await response.json().catch(() => null);
  if (!response.ok) throw new ApiError(data?.error || 'Не удалось выполнить запрос.', response.status);
  return data;
}

export function errorMessage(error) {
  if (error.status === 401) return 'Сессия истекла. Войдите снова.';
  if (error.status === 403) return 'Недостаточно прав для этого действия.';
  if (error.status === 409) return error.message.includes('bookings')
    ? 'У отеля есть бронирования. Удалить его нельзя.' : 'Этот email уже зарегистрирован.';
  if (error.status === 422) return `Проверьте заполненные поля. ${error.message}`;
  if (error.status >= 500) return 'Сервер временно недоступен. Попробуйте позже.';
  return error.message;
}
