import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';

import EventLine from './EventLine.svelte';

function makeLongText(length: number): string {
  return 'A'.repeat(length);
}

describe('EventLine', () => {
  it('expands long content instead of truncating it', async () => {
    const user = userEvent.setup();
    const longText = makeLongText(320);

    render(EventLine, {
      props: {
        event: { type: 'text', content: longText },
        index: 0,
      },
    });

    expect(screen.getByText('Show more')).toBeInTheDocument();
    await user.click(screen.getByText('Show more'));
    expect(screen.getByText(longText)).toBeInTheDocument();
    expect(screen.getByText('Show less')).toBeInTheDocument();
  });
});
