// skein-canvas.jsx — composes the design canvas with three Skein artboards + Tweaks.

const SK_TWEAKS = /*EDITMODE-BEGIN*/{
  "theme": "dark",
  "shelfStyle": "suggestive",
  "sidebar": "open",
  "scenario": "populated",
  "pageFont": "Source Serif 4"
}/*EDITMODE-END*/;

function App() {
  const [t, setTweak] = useTweaks(SK_TWEAKS);

  return (
    <>
      <DesignCanvas>
        <DCSection
          id="skein-mockups"
          title="Skein"
          subtitle="A bookshelf, a desk, and an assistant at your right elbow. GNOME-style desktop · dark theme."
        >
          <DCArtboard id="populated" label="Populated · Split view · Dark" width={1480} height={920}>
            <SkeinWindow
              theme="dark"
              shelfStyle="suggestive"
              sidebar="open"
              scenario="populated"
              vault="Field Notes"
              pageFont={t.pageFont}
            />
          </DCArtboard>

          <DCArtboard id="dragging" label="Drag-to-insert · mid-drag" width={1480} height={920}>
            <SkeinWindow
              theme="dark"
              shelfStyle="suggestive"
              sidebar="open"
              scenario="dragging"
              vault="Field Notes"
              pageFont={t.pageFont}
            />
          </DCArtboard>

          <DCArtboard id="empty" label="Empty state · loose pages on the desk" width={1480} height={920}>
            <SkeinWindow
              theme="dark"
              shelfStyle="suggestive"
              sidebar="open"
              scenario="empty"
              vault="Field Notes"
              pageFont={t.pageFont}
            />
          </DCArtboard>
        </DCSection>

        <DCSection
          id="shelf-variations"
          title="Shelf realism — pick one"
          subtitle="Same window, three takes on how literal the bookshelf should look."
        >
          <DCArtboard id="abstract" label="A · Abstract — flat spines on a chrome bar" width={1480} height={920}>
            <SkeinWindow theme="dark" shelfStyle="abstract" sidebar="open" scenario="populated" pageFont={t.pageFont} />
          </DCArtboard>
          <DCArtboard id="suggestive" label="B · Suggestive — soft wood, flat spines" width={1480} height={920}>
            <SkeinWindow theme="dark" shelfStyle="suggestive" sidebar="open" scenario="populated" pageFont={t.pageFont} />
          </DCArtboard>
          <DCArtboard id="tactile" label="C · Tactile — visible plank, deeper shadow" width={1480} height={920}>
            <SkeinWindow theme="dark" shelfStyle="tactile" sidebar="open" scenario="populated" pageFont={t.pageFont} />
          </DCArtboard>
        </DCSection>

        <DCSection
          id="live-preview"
          title="Live preview"
          subtitle="This artboard tracks the Tweaks panel — flip themes, sidebar modes, fonts."
        >
          <DCArtboard id="live" label="Live · driven by Tweaks" width={1480} height={920}>
            <SkeinWindow
              theme={t.theme}
              shelfStyle={t.shelfStyle}
              sidebar={t.sidebar}
              scenario={t.scenario}
              vault="Field Notes"
              pageFont={t.pageFont}
            />
          </DCArtboard>
        </DCSection>
      </DesignCanvas>

      <TweaksPanel>
        <TweakSection label="Theme" />
        <TweakRadio
          label="Mode"
          value={t.theme}
          options={['dark', 'light']}
          onChange={(v) => setTweak('theme', v)}
        />

        <TweakSection label="Bookshelf" />
        <TweakRadio
          label="Realism"
          value={t.shelfStyle}
          options={['abstract', 'suggestive', 'tactile']}
          onChange={(v) => setTweak('shelfStyle', v)}
        />

        <TweakSection label="Sidebar" />
        <TweakRadio
          label="Mode"
          value={t.sidebar}
          options={['open', 'collapsed', 'hidden']}
          onChange={(v) => setTweak('sidebar', v)}
        />

        <TweakSection label="Desk" />
        <TweakRadio
          label="Scenario"
          value={t.scenario}
          options={['populated', 'dragging', 'empty']}
          onChange={(v) => setTweak('scenario', v)}
        />

        <TweakSection label="Page typography" />
        <TweakSelect
          label="Serif"
          value={t.pageFont}
          options={['Source Serif 4', 'Iowan Old Style', 'Spectral', 'Lora']}
          onChange={(v) => setTweak('pageFont', v)}
        />
      </TweaksPanel>
    </>
  );
}

ReactDOM.createRoot(document.getElementById('root')).render(<App />);
