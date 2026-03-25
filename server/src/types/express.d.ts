declare namespace Express {
  interface Request {
    user?: {
      username: string;
      user_role: number;
    };
  }
}