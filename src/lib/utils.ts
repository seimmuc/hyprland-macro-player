export function sleep(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

export function wordToTitleCase(word: string): string {
  return word.charAt(0).toUpperCase() + word.slice(1);
}
