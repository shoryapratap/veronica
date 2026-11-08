import React, { useEffect, useRef, useState, useCallback } from "react";
import * as maplibregl from "maplibre-gl";
import "maplibre-gl/dist/maplibre-gl.css";
import {
  Globe,
  Compass,
  Plus,
  Minus,
  RotateCw,
  Eye,
  Crosshair,
} from "lucide-react";

// CARTO Basemaps API Authentication Key
const CARTO_API_KEY = "cb1_54jj_1_350f0010f566f4143929adc8";

// Map style configurations adhering strictly to the obsidian & silver design charter
const MAP_STYLES = {
  tactical: {
    id: "tactical",
    name: "Tactical Dark",
    filterClass: "contrast-115 brightness-95",
    style: {
      version: 8 as const,
      projection: { type: "globe" as const },
      sources: {
        "carto-dark": {
          type: "raster" as const,
          tiles: [
            `https://a.basemaps.cartocdn.com/rastertiles/dark_all/{z}/{x}/{y}@2x.png?key=${CARTO_API_KEY}`,
            `https://b.basemaps.cartocdn.com/rastertiles/dark_all/{z}/{x}/{y}@2x.png?key=${CARTO_API_KEY}`,
            `https://c.basemaps.cartocdn.com/rastertiles/dark_all/{z}/{x}/{y}@2x.png?key=${CARTO_API_KEY}`,
          ],
          tileSize: 256,
          maxzoom: 19,
          attribution: "CartoDB Dark Matter",
        },
      },
      layers: [
        {
          id: "carto-dark-layer",
          type: "raster" as const,
          source: "carto-dark",
          minzoom: 0,
          maxzoom: 19,
        },
      ],
      sky: {
        "atmosphere-blend": [
          "interpolate",
          ["linear"],
          ["zoom"],
          0, 0.9,
          5, 0.4,
          7, 0,
        ],
      },
    },
  },
  recon: {
    id: "recon",
    name: "FLIR Recon",
    filterClass: "grayscale-[90%] contrast-[130%] brightness-[90%]",
    style: {
      version: 8 as const,
      projection: { type: "globe" as const },
      sources: {
        satellite: {
          type: "raster" as const,
          tiles: [
            "https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}",
          ],
          tileSize: 256,
          maxzoom: 19,
          attribution: "Esri Satellite FLIR",
        },
      },
      layers: [
        {
          id: "satellite-layer",
          type: "raster" as const,
          source: "satellite",
          minzoom: 0,
          maxzoom: 19,
        },
      ],
      sky: {
        "atmosphere-blend": [
          "interpolate",
          ["linear"],
          ["zoom"],
          0, 1,
          5, 0.7,
          7, 0,
        ],
      },
    },
  },
  satellite: {
    id: "satellite",
    name: "True Color",
    filterClass: "contrast-105",
    style: {
      version: 8 as const,
      projection: { type: "globe" as const },
      sources: {
        satellite: {
          type: "raster" as const,
          tiles: [
            "https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}",
          ],
          tileSize: 256,
          maxzoom: 19,
          attribution: "Esri World Imagery",
        },
      },
      layers: [
        {
          id: "satellite-layer",
          type: "raster" as const,
          source: "satellite",
          minzoom: 0,
          maxzoom: 19,
        },
      ],
    },
  },
};

type StyleMode = keyof typeof MAP_STYLES;

export const EarthGlobe: React.FC = () => {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const mapRef = useRef<maplibregl.Map | null>(null);

  // Default to Tactical Dark (Monochrome / Obsidian Silver)
  const [activeStyle, setActiveStyle] = useState<StyleMode>("tactical");
  const [autoRotate, setAutoRotate] = useState(true);
  const [isTilted, setIsTilted] = useState(false);
  const [showReticle, setShowReticle] = useState(true);

  // Live Telemetry
  const [zoom, setZoom] = useState<number>(1.3);
  const [bearing, setBearing] = useState<number>(0);
  const [pitch, setPitch] = useState<number>(0);

  const isInteractingRef = useRef(false);
  const autoRotateRef = useRef(autoRotate);
  autoRotateRef.current = autoRotate;

  // Initialize Map
  useEffect(() => {
    if (!containerRef.current || mapRef.current) return;

    const map = new maplibregl.Map({
      container: containerRef.current,
      style: MAP_STYLES[activeStyle].style as unknown as maplibregl.StyleSpecification,
      center: [20, 15],
      zoom: 1.3,
      minZoom: 0.5,
      maxZoom: 18.5,
      attributionControl: false,
    });

    mapRef.current = map;

    map.on("load", () => {
      try {
        map.setProjection({ type: "globe" });
      } catch (err) {
        console.warn("setProjection fallback:", err);
      }
    });

    const updateTelemetry = () => {
      setZoom(map.getZoom());
      setBearing(Math.round(map.getBearing()));
      setPitch(Math.round(map.getPitch()));
    };

    map.on("move", updateTelemetry);

    map.on("mousedown", () => {
      isInteractingRef.current = true;
    });
    map.on("mouseup", () => {
      isInteractingRef.current = false;
    });
    map.on("touchstart", () => {
      isInteractingRef.current = true;
    });
    map.on("touchend", () => {
      isInteractingRef.current = false;
    });
    map.on("wheel", () => {
      isInteractingRef.current = true;
      setTimeout(() => {
        isInteractingRef.current = false;
      }, 1500);
    });

    // Auto-rotation loop
    let animId: number;
    const rotateGlobe = () => {
      if (
        mapRef.current &&
        autoRotateRef.current &&
        !isInteractingRef.current &&
        mapRef.current.getZoom() < 4
      ) {
        const center = mapRef.current.getCenter();
        center.lng += 0.1;
        mapRef.current.jumpTo({ center });
      }
      animId = requestAnimationFrame(rotateGlobe);
    };
    animId = requestAnimationFrame(rotateGlobe);

    // Resize observer
    const ro = new ResizeObserver(() => {
      map.resize();
    });
    ro.observe(containerRef.current);

    return () => {
      cancelAnimationFrame(animId);
      ro.disconnect();
      map.remove();
      mapRef.current = null;
    };
  }, []);

  // Handle Style Switching
  const switchStyle = useCallback((styleKey: StyleMode) => {
    setActiveStyle(styleKey);
    const map = mapRef.current;
    if (!map) return;

    map.setStyle(MAP_STYLES[styleKey].style as unknown as maplibregl.StyleSpecification);
    map.once("styledata", () => {
      try {
        map.setProjection({ type: "globe" });
      } catch (e) {
        console.warn("re-setProjection:", e);
      }
    });
  }, []);

  // Controls
  const handleZoomIn = () => {
    mapRef.current?.zoomIn({ duration: 300 });
  };

  const handleZoomOut = () => {
    mapRef.current?.zoomOut({ duration: 300 });
  };

  const handleResetNorth = () => {
    mapRef.current?.resetNorthPitch({ duration: 600 });
    setIsTilted(false);
  };

  const handleToggleTilt = () => {
    const map = mapRef.current;
    if (!map) return;
    const nextTilt = !isTilted;
    setIsTilted(nextTilt);
    map.easeTo({
      pitch: nextTilt ? 55 : 0,
      duration: 600,
    });
  };

  const altitudeKm = Math.max(1, Math.round(12800 / Math.pow(2, zoom)));

  return (
    <div className="flex flex-col h-full bg-zinc-950/70 backdrop-blur-xl border border-zinc-800/60 rounded-xl overflow-hidden shadow-2xl relative select-none">
      {/* 1. Futuristic Header Bar with Sensor Status */}
      <div className="flex items-center justify-between px-3.5 py-2.5 border-b border-zinc-800/60 shrink-0 bg-zinc-950/60">
        <div className="flex items-center gap-2">
          <Globe className="w-3.5 h-3.5 text-zinc-300 animate-pulse" />
          <span className="text-xs font-mono font-medium text-zinc-200 tracking-wider uppercase">
            Orbital Recon // 3D
          </span>
        </div>

        {/* Style Selector Tabs (Silver / Zinc Aesthetic) */}
        <div className="flex items-center bg-zinc-900/90 border border-zinc-800/80 p-0.5 rounded-lg shadow-inner">
          {(["tactical", "recon", "satellite"] as StyleMode[]).map((mode) => (
            <button
              key={mode}
              onClick={() => switchStyle(mode)}
              className={`px-2 py-0.5 text-[10px] font-mono rounded-md transition-all uppercase tracking-tight ${
                activeStyle === mode
                  ? "bg-zinc-800 text-zinc-100 shadow-sm border border-zinc-700/60 font-medium"
                  : "text-zinc-500 hover:text-zinc-300"
              }`}
            >
              {mode === "tactical" ? "TAC-DARK" : mode === "recon" ? "FLIR" : "COLOR"}
            </button>
          ))}
        </div>
      </div>

      {/* 2. Interactive Globe Canvas & Holographic HUD Container */}
      <div className="flex-1 relative w-full h-full min-h-0 bg-black overflow-hidden">
        {/* Soft Metallic Atmospheric Glow behind the Globe when in orbit */}
        {zoom < 3 && (
          <div className="absolute inset-0 pointer-events-none flex items-center justify-center z-0">
            <div className="w-72 h-72 rounded-full bg-gradient-radial from-white/[0.07] via-zinc-400/[0.02] to-transparent blur-2xl" />
          </div>
        )}

        {/* MapLibre WebGL Canvas Container with Dynamic Filter */}
        <div
          ref={containerRef}
          className={`absolute inset-0 w-full h-full transition-all duration-500 ${MAP_STYLES[activeStyle].filterClass}`}
        />

        {/* Holographic Sci-Fi HUD Layer (Corner Brackets & Frame Details) */}
        <div className="absolute inset-0 pointer-events-none z-10 p-2.5 flex flex-col justify-between">
          {/* Top Row Framing */}
          <div className="flex justify-between items-start">
            <div className="w-3 h-3 border-t-2 border-l-2 border-zinc-600/70" />
            <div className="w-3 h-3 border-t-2 border-r-2 border-zinc-600/70" />
          </div>

          {/* Bottom Row Framing */}
          <div className="flex justify-between items-end">
            <div className="w-3 h-3 border-b-2 border-l-2 border-zinc-600/70" />
            <div className="w-3 h-3 border-b-2 border-r-2 border-zinc-600/70" />
          </div>
        </div>

        {/* Center Targeting Reticle (Toggleable) */}
        {showReticle && (
          <div className="absolute inset-0 pointer-events-none flex items-center justify-center z-10">
            <div className="relative flex items-center justify-center">
              <Crosshair className="w-7 h-7 text-zinc-400/40" />
              <div className="absolute w-12 h-12 rounded-full border border-zinc-500/15" />
            </div>
          </div>
        )}

        {/* Top-Right: Holographic Control Deck */}
        <div className="absolute top-3 right-3 z-20 flex flex-col gap-1.5">
          {/* Zoom In */}
          <button
            onClick={handleZoomIn}
            title="Zoom In"
            className="p-1.5 rounded-lg bg-zinc-950/80 hover:bg-zinc-900 border border-zinc-800/80 hover:border-zinc-700 text-zinc-300 hover:text-white backdrop-blur-xl shadow-lg transition-all active:scale-95"
          >
            <Plus className="w-3.5 h-3.5" />
          </button>

          {/* Zoom Out */}
          <button
            onClick={handleZoomOut}
            title="Zoom Out"
            className="p-1.5 rounded-lg bg-zinc-950/80 hover:bg-zinc-900 border border-zinc-800/80 hover:border-zinc-700 text-zinc-300 hover:text-white backdrop-blur-xl shadow-lg transition-all active:scale-95"
          >
            <Minus className="w-3.5 h-3.5" />
          </button>

          {/* Reset North Compass */}
          <button
            onClick={handleResetNorth}
            title="Reset Bearing (North)"
            className="p-1.5 rounded-lg bg-zinc-950/80 hover:bg-zinc-900 border border-zinc-800/80 hover:border-zinc-700 text-zinc-300 hover:text-white backdrop-blur-xl shadow-lg transition-all active:scale-95"
          >
            <Compass
              className="w-3.5 h-3.5 transition-transform"
              style={{ transform: `rotate(${-bearing}deg)` }}
            />
          </button>

          {/* 3D Horizon Tilt */}
          <button
            onClick={handleToggleTilt}
            title="Toggle 3D Horizon Pitch"
            className={`p-1.5 rounded-lg border backdrop-blur-xl shadow-lg transition-all active:scale-95 ${
              isTilted
                ? "bg-zinc-800 border-zinc-500 text-white"
                : "bg-zinc-950/80 hover:bg-zinc-900 border-zinc-800/80 hover:border-zinc-700 text-zinc-300"
            }`}
          >
            <Eye className="w-3.5 h-3.5" />
          </button>

          {/* Auto-Rotate Toggle */}
          <button
            onClick={() => setAutoRotate((prev) => !prev)}
            title="Toggle Planetary Auto-Orbit"
            className={`p-1.5 rounded-lg border backdrop-blur-xl shadow-lg transition-all active:scale-95 ${
              autoRotate
                ? "bg-zinc-800 border-zinc-500 text-white"
                : "bg-zinc-950/80 hover:bg-zinc-900 border-zinc-800/80 hover:border-zinc-700 text-zinc-500"
            }`}
          >
            <RotateCw
              className={`w-3.5 h-3.5 ${autoRotate ? "animate-spin" : ""}`}
              style={{ animationDuration: "12s" }}
            />
          </button>

          {/* Reticle Toggle */}
          <button
            onClick={() => setShowReticle((prev) => !prev)}
            title="Toggle Reticle HUD"
            className={`p-1.5 rounded-lg border backdrop-blur-xl shadow-lg transition-all active:scale-95 ${
              showReticle
                ? "bg-zinc-800 border-zinc-500 text-white"
                : "bg-zinc-950/80 hover:bg-zinc-900 border-zinc-800/80 hover:border-zinc-700 text-zinc-500"
            }`}
          >
            <Crosshair className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      {/* 3. Bottom Telemetry (Luxury Silver / Glassmorphic) */}
      <div className="px-3.5 py-2 border-t border-zinc-800/60 bg-zinc-950/80 shrink-0 flex items-center justify-between font-mono text-[10px] text-zinc-500">
        <div className="flex items-center gap-3">
          <span>ALT: ~{altitudeKm.toLocaleString()} KM</span>
          <span>ZOOM: {zoom.toFixed(2)}x</span>
        </div>
        <div className="flex items-center gap-2">
          <span>TILT: {pitch}°</span>
        </div>
      </div>
    </div>
  );
};
