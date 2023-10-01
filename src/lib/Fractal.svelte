<script lang="ts">
	import { onMount } from 'svelte';
	import * as THREE from 'three';

	export let noise: number[] = [];
	export let expression: string;

	$: filledNoise = (() => {
		let filled = [...noise];
		let missing = 16 - noise.length;
		for (let i = 0; i < missing; ++i) {
			filled.push(0);
		}
		return filled;
	})();

	var camera, scene, renderer;
	var geometry, material, mesh;
	var uniforms;

	let div: HTMLDivElement;

	function getMaterial() {
		return new THREE.ShaderMaterial({
			fragmentShader: `
        const float PI = 3.141596;
        #define cx_mul(a, b) vec2(a.x*b.x-a.y*b.y, a.x*b.y+a.y*b.x)
        #define cx_div(a, b) vec2(((a.x*b.x+a.y*b.y)/(b.x*b.x+b.y*b.y)),((a.y*b.x-a.x*b.y)/(b.x*b.x+b.y*b.y)))
        #define cx_modulus(a) length(a)
        #define cx_conj(a) vec2(a.x, -a.y)
        #define cx_arg(a) atan(a.y, a.x)
        #define cx_sin(a) vec2(sin(a.x) * cosh(a.y), cos(a.x) * sinh(a.y))
        #define cx_cos(a) vec2(cos(a.x) * cosh(a.y), -sin(a.x) * sinh(a.y))

        vec2 cx_sqrt(vec2 a) {
            float r = length(a);
            float rpart = sqrt(0.5*(r+a.x));
            float ipart = sqrt(0.5*(r-a.x));
            if (a.y < 0.0) ipart = -ipart;
            return vec2(rpart,ipart);
        }

        vec2 cx_tan(vec2 a) {return cx_div(cx_sin(a), cx_cos(a)); }

        vec2 cx_log(vec2 a) {
            float rpart = sqrt((a.x*a.x)+(a.y*a.y));
            float ipart = atan(a.y,a.x);
            if (ipart > PI) ipart=ipart-(2.0*PI);
            return vec2(log(rpart),ipart);
        }

        vec2 cx_mobius(vec2 a) {
            vec2 c1 = a - vec2(1.0,0.0);
            vec2 c2 = a + vec2(1.0,0.0);
            return cx_div(c1, c2);
        }

        vec2 cx_z_plus_one_over_z(vec2 a) {
            return a + cx_div(vec2(1.0,0.0), a);
        }

        vec2 cx_z_squared_plus_c(vec2 z, vec2 c) {
            return cx_mul(z, z) + c;
        }

        vec2 cx_sin_of_one_over_z(vec2 z) {
            return cx_sin(cx_div(vec2(1.0,0.0), z));
        }

        ////////////////////////////////////////////////////////////
        // end Complex Number math by julesb
        ////////////////////////////////////////////////////////////

        // My own additions to complex number math
        #define cx_sub(a, b) vec2(a.x - b.x, a.y - b.y)
        #define cx_add(a, b) vec2(a.x + b.x, a.y + b.y)
        #define cx_abs(a) length(a)
        vec2 cx_to_polar(vec2 a) {
            float phi = atan(a.y / a.x);
            float r = length(a);
            return vec2(r, phi); 
        }
            
        // Complex power
        // Let z = r(cos θ + i sin θ)
        // Then z^n = r^n (cos nθ + i sin nθ)
        vec2 cx_pow(vec2 a, float n) {
            float angle = atan(a.y, a.x);
            float r = length(a);
            float real = pow(r, n) * cos(n*angle);
            float im = pow(r, n) * sin(n*angle);
            return vec2(real, im);
        }

        float cabs (vec2 z) {
return length(z);
}

// complex conjugate of z
vec2 cconj (vec2 z) {
return vec2(z.x, -z.y);
}

// a times b
vec2 cmul (vec2 a, vec2 b) {
return vec2(a.x * b.x - a.y * b.y, a.x * b.y + b.x * a.y);
}

// a divided by b
vec2 cdiv (vec2 a, vec2 b) {
float den = dot(b, b);
return vec2(
    (a.x * b.x + a.y * b.y) / den,
    (a.x * b.x - a.y * b.y) / den
);
}

// exponential of z
vec2 cexp (vec2 z) {
return exp(z.x) * vec2(cos(z.y), sin(z.y));
}

// principal value of logarithm of z
vec2 clog (vec2 z) {
return vec2(log(cabs(z)), atan(z.y, z.x));
}

// principal value of z to the a power
vec2 cpow (vec2 z, vec2 a) {
return cexp(cmul(a, clog(z)));
}

// cosine of z
vec2 ccos (vec2 z) {
return (cexp(vec2(-z.y, z.x)) + cexp(vec2(z.y, -z.x))) / 2.0;
}

// sine of z
vec2 csin (vec2 z) {
vec2 t = (cexp(vec2(-z.y, z.x)) - cexp(vec2(z.y, -z.x))) / 2.0;
return vec2(-t.y, -t.x);
}

// tangent of z
vec2 ctan (vec2 z) {
return cdiv(csin(z), ccos(z));
}

// hyperbolic cosine of z
vec2 ccosh (vec2 z) {
return (cexp(z) + cexp(-z)) / 2.0;
}

// hyperbolic sine of z
vec2 csinh (vec2 z) {
return (cexp(z) - cexp(-z)) / 2.0;
}

// hyperbolic tangent of z
vec2 ctanh (vec2 z) {
return cdiv(csinh(z), ccosh(z));
}

vec3 hsv2rgb(vec3 c)
{
vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}
        uniform vec2 res; 

        struct Noise { float val; };
        uniform Noise noiseList[16];
        uniform int len;

        void main() {
            vec2 sum = vec2(0.0);
            vec2 uv = gl_FragCoord.xy / res - vec2(0.5);

            float noise = 0.0;
            for (int i = 0; i < 4; i++) {
                for (int j = 0; j < 4; j++) {
                    noise += noiseList[i * 4 + j].val * exp(-pow(distance(
                        uv, res / 3.0 * vec2(i, j)
                    ), 2.0));
                }    
            }

            for (int i = 1; i <= 1000; i++) {
                sum += ${expression};
                if (length(sum) > 1000.0) {
                    vec2 polar = cx_to_polar(sum);
                    float iter_cf = float(i) / 1000.0 - 0.5;
                    gl_FragColor = vec4(hsv2rgb(vec3(iter_cf, 1.0 - iter_cf, 1.0)), 1.0);
                    return;
                }
            }
            vec2 polar = cx_to_polar(sum);
            gl_FragColor = vec4(hsv2rgb(vec3(polar.y / (2.0 * PI), polar.x * 3.0, 1.0)), 1.0);
        }  
        `,
			uniforms: {
				res: { value: new THREE.Vector2(1024, 1024) },
				noiseList: { value: filledNoise.map((val) => ({ val })) }
			}
		});
	}

	function init() {
		setup();

		geometry = new THREE.PlaneGeometry(2, 2);
		material = getMaterial();
		mesh = new THREE.Mesh(geometry, material);
		scene.add(mesh);
		renderer.render(scene, camera);
	}

	function setup() {
		camera = new THREE.OrthographicCamera(-1, 1, 1, -1, -1, 1);
		scene = new THREE.Scene();
		renderer = new THREE.WebGLRenderer({ antialias: false, precision: 'highp' });
		renderer.setSize(div.clientWidth, div.clientHeight);
		div.appendChild(renderer.domElement);
	}
	let mounted = false;
	onMount(() => {
		mounted = true;
		init();
	});

	$: if (mounted) {
		expression;
		// filledNoise;
		mesh.material = getMaterial();
		renderer.render(scene, camera);
	}
</script>

<div class="aspect-square grow" bind:this={div} />
