<script lang="ts">
  import Env from '$lib/components/Env.svelte';
  import StandardInput from '$lib/components/StandardInput.svelte';
  import { Io } from '$lib/io.svelte';

  const io = new Io('Challenge2', { commands: 'skip' });
</script>

<Env {io}>
  {#snippet inputView()}
    <StandardInput analysis="Challenge2" {io} />
  {/snippet}
  {#snippet outputView({ output, referenceOutput })}{/snippet}
</Env>
