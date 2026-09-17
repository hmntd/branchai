export interface GitAccount {
  id: string;
  name: string;
  provider: 'github' | 'gitlab' | 'bitbucket' | 'custom';
  username: string;
  email: string;
  avatar_url?: string;
  auth_type: 'ssh' | 'pat' | 'https';
  ssh_key_path?: string;
  personal_access_token?: string;
}

export function getProviderLabel(provider: string): string {
  switch (provider.toLowerCase()) {
    case 'github':
      return 'GitHub';
    case 'gitlab':
      return 'GitLab';
    case 'bitbucket':
      return 'Bitbucket';
    default:
      return 'Git';
  }
}

export function getAccountInitials(account: GitAccount): string {
  if (account.name && account.name.trim()) {
    const parts = account.name.trim().split(' ');
    if (parts.length >= 2) {
      return (parts[0][0] + parts[1][0]).toUpperCase();
    }
    return account.name.substring(0, 2).toUpperCase();
  }
  if (account.username && account.username.trim()) {
    return account.username.substring(0, 2).toUpperCase();
  }
  return 'GA';
}
